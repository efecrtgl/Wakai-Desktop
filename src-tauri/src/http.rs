//! Windows'un yerleÅŸik WinHTTP yÄ±ÄŸÄ±nÄ±yla basit HTTPS GET istemcisi.
//!
//! Neden WinHTTP: Uygulamaya ayrÄ± bir TLS kÃ¼tÃ¼phanesi gÃ¶mmek gerekmez; sertifika
//! doÄŸrulamasÄ± Windows sertifika deposu ve SChannel ile yapÄ±lÄ±r. (WinRT `HttpClient`
//! geliÅŸtirme makinesinde Let's Encrypt sertifikalÄ± sunucularda `0x80072F8F` ile
//! baÅŸarÄ±sÄ±z oldu; WinHTTP aynÄ± sunucularda sorunsuz Ã§alÄ±ÅŸtÄ±.)

use crate::error::{AppError, AppResult};
use windows::Win32::Networking::WinHttp::{
    INTERNET_DEFAULT_HTTPS_PORT, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE,
    WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable,
    WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
    WinHttpSetTimeouts,
};
use windows::core::{HSTRING, PCWSTR};

/// Ä°stek baÅŸÄ±na toplam yanÄ±t sÄ±nÄ±rÄ± (hava durumu yanÄ±tlarÄ± birkaÃ§ KB'dir).
const MAX_BODY: usize = 2 * 1024 * 1024;

struct Handle(*mut core::ffi::c_void);

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = WinHttpCloseHandle(self.0);
            }
        }
    }
}

fn check(h: *mut core::ffi::c_void, what: &str) -> AppResult<Handle> {
    if h.is_null() {
        Err(AppError::Other(format!(
            "{what}: {}",
            windows::core::Error::from_thread().message()
        )))
    } else {
        Ok(Handle(h))
    }
}

/// "https://host/yol?sorgu" â†’ (host, "/yol?sorgu"). YalnÄ±zca HTTPS kabul edilir.
fn split_url(url: &str) -> AppResult<(&str, &str)> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| AppError::invalid("YalnÄ±zca HTTPS adresleri desteklenir."))?;
    match rest.find('/') {
        Some(i) => Ok((&rest[..i], &rest[i..])),
        None => Ok((rest, "/")),
    }
}

pub fn get(url: &str) -> AppResult<String> {
    let (host, path) = split_url(url)?;
    let fail = |what: &str| {
        AppError::Other(format!(
            "{what}: {}",
            windows::core::Error::from_thread().message()
        ))
    };
    unsafe {
        let session = check(
            WinHttpOpen(
                &HSTRING::from(concat!("WakaiDesktop/", env!("CARGO_PKG_VERSION"))),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            ),
            "BaÄŸlantÄ± aÃ§Ä±lamadÄ±",
        )?;
        // Ã‡Ã¶zÃ¼mleme, baÄŸlanma, gÃ¶nderme, alma (ms)
        let _ = WinHttpSetTimeouts(session.0, 5_000, 10_000, 10_000, 15_000);
        let connect = check(
            WinHttpConnect(
                session.0,
                &HSTRING::from(host),
                INTERNET_DEFAULT_HTTPS_PORT,
                0,
            ),
            "Sunucuya baÄŸlanÄ±lamadÄ±",
        )?;
        let request = check(
            WinHttpOpenRequest(
                connect.0,
                &HSTRING::from("GET"),
                &HSTRING::from(path),
                PCWSTR::null(),
                PCWSTR::null(),
                std::ptr::null_mut(),
                WINHTTP_OPEN_REQUEST_FLAGS(WINHTTP_FLAG_SECURE.0),
            ),
            "Ä°stek oluÅŸturulamadÄ±",
        )?;
        WinHttpSendRequest(request.0, None, None, 0, 0, 0)
            .map_err(|_| fail("Ä°stek gÃ¶nderilemedi"))?;
        WinHttpReceiveResponse(request.0, std::ptr::null_mut())
            .map_err(|_| fail("YanÄ±t alÄ±namadÄ±"))?;

        let mut status: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut u32 as *mut _),
            &mut size,
            std::ptr::null_mut(),
        )
        .map_err(|_| fail("YanÄ±t durumu okunamadÄ±"))?;

        let mut body = Vec::new();
        loop {
            let mut available = 0u32;
            WinHttpQueryDataAvailable(request.0, &mut available)
                .map_err(|_| fail("YanÄ±t okunamadÄ±"))?;
            if available == 0 {
                break;
            }
            let start = body.len();
            body.resize(start + available as usize, 0);
            let mut read = 0u32;
            WinHttpReadData(
                request.0,
                body[start..].as_mut_ptr() as *mut _,
                available,
                &mut read,
            )
            .map_err(|_| fail("YanÄ±t okunamadÄ±"))?;
            body.truncate(start + read as usize);
            if body.len() > MAX_BODY {
                return Err(AppError::Other("YanÄ±t beklenenden bÃ¼yÃ¼k.".into()));
            }
        }
        if !(200..300).contains(&status) {
            return Err(AppError::Other(format!(
                "Sunucu {status} durum kodu dÃ¶ndÃ¼rdÃ¼."
            )));
        }
        String::from_utf8(body).map_err(|_| AppError::Other("YanÄ±t UTF-8 deÄŸil.".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_splitting() {
        assert_eq!(
            split_url("https://api.example.com/v1/x?a=1").unwrap(),
            ("api.example.com", "/v1/x?a=1")
        );
        assert_eq!(
            split_url("https://example.com").unwrap(),
            ("example.com", "/")
        );
        assert!(split_url("http://example.com/").is_err());
    }

    /// GerÃ§ek aÄŸ isteÄŸi: `cargo test real_https -- --ignored`
    #[test]
    #[ignore]
    fn real_https() {
        let body =
            get("https://geocoding-api.open-meteo.com/v1/search?name=Izmir&count=1&format=json");
        println!("{body:?}");
        assert!(body.unwrap().contains("results"));
    }
}
