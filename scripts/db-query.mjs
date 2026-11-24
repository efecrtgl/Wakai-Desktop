// Geliştirme aracı: uygulama veritabanında salt okunur sorgu çalıştırır.
//   node scripts/db-query.mjs "SELECT * FROM events"
import { DatabaseSync } from 'node:sqlite';
const db = new DatabaseSync(`${process.env.APPDATA}/com.wakai.desktop/wakai.db`, { readOnly: true });
console.table(db.prepare(process.argv[2]).all());
