// Kedi arayüzünün Türkçe etiketleri.
export const PERSONALITIES = {
  playful: { name: 'Oyuncu', desc: 'Sık koşar ve oynar, pencerelere zıplamayı sever; çabuk acıkır.' },
  calm: { name: 'Sakin', desc: 'Uzun uzun oturur, az koşar; ihtiyaçları yavaş değişir.' },
  curious: { name: 'Meraklı', desc: 'Etrafı keşfeder, sık sık bakınır ve pencerelere tırmanır.' },
  sleepy: { name: 'Uykucu', desc: 'Sık ve uzun uyur.' },
  affectionate: { name: 'Sevecen', desc: 'Sana sık yaklaşır; sevilmeyi çok sever, yalnız kalınca üzülür.' },
  independent: { name: 'Bağımsız', desc: 'Kendi başına takılır, nadiren yaklaşır.' },
};

export const NEEDS = [
  { key: 'satiety', label: 'Tokluk' },
  { key: 'energy', label: 'Enerji' },
  { key: 'happiness', label: 'Mutluluk' },
  { key: 'rest', label: 'Uyku' },
];

export const ACTIVITY_LEVELS = [
  { value: 'paused', label: 'Duraklatılmış (yerinde durur)' },
  { value: 'low', label: 'Düşük' },
  { value: 'normal', label: 'Normal' },
  { value: 'high', label: 'Yüksek' },
];

export const LOCK_HINTS = {
  first_friend: 'İlk kedini sahiplen',
  feed_10: '10 kez besle',
  pet_30: '30 kez sev',
  play_10: '10 kez oyna',
  streak_3: '3 gün üst üste besle',
  happy_cat: 'Bir kediyi çok mutlu et',
  collector: '4 farklı cins sahiplen',
};

/** Kedinin genel ruh hâli (kart başlığında gösterilir). */
export function mood(needs) {
  if (needs.rest < 25) return 'Uykulu';
  if (needs.satiety < 30) return 'Acıkmış';
  if (needs.happiness < 35) return 'İlgi istiyor';
  if (needs.energy < 25) return 'Yorgun';
  if (needs.happiness > 85) return 'Çok mutlu';
  return 'Keyifli';
}
