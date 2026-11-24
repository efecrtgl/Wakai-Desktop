// WMO hava durumu kodları → Türkçe açıklama, piksel simge ve kedi pozu.
const TABLE = [
  [[0], 'Açık', 'sun', 'happy'],
  [[1], 'Çoğunlukla açık', 'sun', 'happy'],
  [[2], 'Parçalı bulutlu', 'partly', 'sitting'],
  [[3], 'Kapalı', 'cloud', 'relaxed'],
  [[45, 48], 'Sisli', 'fog', 'curious'],
  [[51, 53, 55], 'Çisenti', 'rain', 'relaxed'],
  [[56, 57], 'Donan çisenti', 'rain', 'relaxed'],
  [[61], 'Hafif yağmur', 'rain', 'relaxed'],
  [[63], 'Yağmurlu', 'rain', 'relaxed'],
  [[65], 'Şiddetli yağmur', 'rain', 'sleeping'],
  [[66, 67], 'Donan yağmur', 'rain', 'sleeping'],
  [[71], 'Hafif kar', 'snow', 'curious'],
  [[73], 'Karlı', 'snow', 'curious'],
  [[75], 'Yoğun kar', 'snow', 'sleeping'],
  [[77], 'Kar taneleri', 'snow', 'curious'],
  [[80, 81], 'Sağanak', 'rain', 'relaxed'],
  [[82], 'Şiddetli sağanak', 'storm', 'sleeping'],
  [[85, 86], 'Kar sağanağı', 'snow', 'sleeping'],
  [[95], 'Gök gürültülü fırtına', 'storm', 'sleeping'],
  [[96, 99], 'Dolulu fırtına', 'storm', 'sleeping'],
];

const MAP = new Map(TABLE.flatMap(([codes, label, icon, cat]) => codes.map((c) => [c, { label, icon, cat }])));

export function describe(code, isDay = true) {
  const d = MAP.get(code) ?? { label: 'Bilinmiyor', icon: 'cloud', cat: 'curious' };
  // Gece açık havada güneş yerine ay.
  if (!isDay && d.icon === 'sun') return { ...d, icon: 'moon', cat: 'sleeping' };
  return d;
}
