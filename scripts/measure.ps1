<#
.SYNOPSIS
  Wakai Desktop'un kaynak kullanımını ölçer (ana süreç + tüm WebView2 alt süreçleri).

.DESCRIPTION
  Çalışan wakai-desktop.exe sürecini ve ondan türeyen tüm süreçleri bulur, belirtilen süre
  boyunca saniyede bir örnekler ve şunları raporlar:
    - Özel bellek (Private Bytes) toplamı: ortalama / en yüksek / son
    - Çalışma kümesi (Working Set) toplamı: ortalama / son
    - CPU kullanımı: toplam CPU süresi farkından, mantıksal çekirdek sayısına bölünerek (%)
    - Süreç sayısı

.EXAMPLE
  powershell -File scripts\measure.ps1 -Duration 60 -Label "9 widget, boşta"
#>
param(
  [int]$Duration = 60,
  [string]$Label = "ölçüm",
  [string]$OutFile = ""
)

$ErrorActionPreference = 'Stop'
$root = Get-Process wakai-desktop -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $root) { Write-Error "wakai-desktop.exe çalışmıyor." }

function Get-Tree([int]$rootId) {
  $all = Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId
  $ids = [System.Collections.Generic.List[int]]::new(); $ids.Add($rootId)
  $i = 0
  while ($i -lt $ids.Count) {
    foreach ($p in $all) { if ($p.ParentProcessId -eq $ids[$i] -and -not $ids.Contains([int]$p.ProcessId)) { $ids.Add([int]$p.ProcessId) } }
    $i++
  }
  return $ids
}

$cores = [Environment]::ProcessorCount
$samples = @()
$ids = Get-Tree $root.Id
$prevCpu = 0.0; $prevTime = Get-Date
foreach ($id in $ids) { $p = Get-Process -Id $id -ErrorAction SilentlyContinue; if ($p) { $prevCpu += $p.TotalProcessorTime.TotalMilliseconds } }

for ($s = 0; $s -lt $Duration; $s++) {
  Start-Sleep -Seconds 1
  $ids = Get-Tree $root.Id
  $priv = 0; $ws = 0; $cpu = 0.0; $count = 0
  foreach ($id in $ids) {
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    if ($p) { $priv += $p.PrivateMemorySize64; $ws += $p.WorkingSet64; $cpu += $p.TotalProcessorTime.TotalMilliseconds; $count++ }
  }
  $now = Get-Date
  $elapsed = ($now - $prevTime).TotalMilliseconds
  $cpuPct = [math]::Max(0, ($cpu - $prevCpu) / ($elapsed * $cores) * 100)
  $prevCpu = $cpu; $prevTime = $now
  $samples += [pscustomobject]@{ Private = $priv; WorkingSet = $ws; Cpu = $cpuPct; Count = $count }
}

$mb = { param($v) [math]::Round($v / 1MB, 1) }
$result = [ordered]@{
  label            = $Label
  duration_s       = $Duration
  processes        = $samples[-1].Count
  private_avg_mb   = & $mb ($samples | Measure-Object Private -Average).Average
  private_max_mb   = & $mb ($samples | Measure-Object Private -Maximum).Maximum
  private_last_mb  = & $mb $samples[-1].Private
  ws_avg_mb        = & $mb ($samples | Measure-Object WorkingSet -Average).Average
  cpu_avg_pct      = [math]::Round(($samples | Measure-Object Cpu -Average).Average, 3)
  cpu_max_pct      = [math]::Round(($samples | Measure-Object Cpu -Maximum).Maximum, 3)
  logical_cores    = $cores
}
$json = $result | ConvertTo-Json
if ($OutFile) { $json | Out-File -FilePath $OutFile -Encoding utf8 }
$json
