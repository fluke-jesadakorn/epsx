import { Firestore, Timestamp, doc, setDoc, collection, getDocs, deleteDoc } from 'firebase/firestore';
import type { SeedOptions, SeedResult } from '../types';

export abstract class BaseSeeder {
  protected db: Firestore;
  protected options: SeedOptions;

  constructor(db: Firestore, options: SeedOptions = { environment: 'development' }) {
    this.db = db;
    this.options = options;
  }

  abstract get collectionName(): string;
  abstract seed(): Promise<SeedResult>;

  protected log(message: string): void {
    if (this.options.verbose !== false) {
      console.log(`[${this.collectionName}] ${message}`);
    }
  }

  protected async seedCollection<T extends Record<string, any>>(
    collectionName: string,
    data: T[],
    idField?: keyof T
  ): Promise<SeedResult> {
    try {
      this.log(`Seeding ${data.length} documents...`);

      // Clear existing data if force option is set
      if (this.options.force) {
        await this.clearCollection(collectionName);
      }

      let count = 0;
      for (const item of data) {
        const docId = idField ? String(item[idField]) : doc(collection(this.db, collectionName)).id;
        const docRef = doc(this.db, collectionName, docId);
        
        const docData = {
          ...item,
          createdAt: item.createdAt || Timestamp.now(),
          updatedAt: item.updatedAt || Timestamp.now(),
        };

        await setDoc(docRef, docData);
        count++;
      }

      this.log(`✅ Successfully seeded ${count} documents`);
      return {
        success: true,
        collection: collectionName,
        count
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.log(`❌ Error seeding: ${errorMessage}`);
      return {
        success: false,
        collection: collectionName,
        count: 0,
        error: errorMessage
      };
    }
  }

  protected async clearCollection(collectionName: string): Promise<void> {
    this.log(`Clearing existing data from ${collectionName}...`);
    const snapshot = await getDocs(collection(this.db, collectionName));
    
    const deletePromises = snapshot.docs.map(doc => deleteDoc(doc.ref));
    await Promise.all(deletePromises);
    
    this.log(`Cleared ${snapshot.docs.length} existing documents`);
  }

  protected createTimestamp(date?: Date): Timestamp {
    return date ? Timestamp.fromDate(date) : Timestamp.now();
  }

  protected addDays(date: Date, days: number): Date {
    const result = new Date(date);
    result.setDate(result.getDate() + days);
    return result;
  }

  protected addHours(date: Date, hours: number): Date {
    const result = new Date(date);
    result.setHours(result.getHours() + hours);
    return result;
  }

  protected generateId(prefix: string = 'id'): string {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

export class SeedManager {
  private seeders: BaseSeeder[] = [];
  private options: SeedOptions;

  constructor(_db: Firestore, options: SeedOptions = { environment: 'development' }) {
    this.options = options;
  }

  addSeeder(seeder: BaseSeeder): void {
    this.seeders.push(seeder);
  }

  async runAll(): Promise<SeedResult[]> {
    console.log(`🌱 Starting seed process for ${this.options.environment} environment...\n`);
    
    const results: SeedResult[] = [];
    
    for (const seeder of this.seeders) {
      // Skip if specific collections are specified and this isn't one of them
      if (this.options.collections && !this.options.collections.includes(seeder.collectionName)) {
        continue;
      }

      console.log(`📦 Seeding ${seeder.collectionName}...`);
      const result = await seeder.seed();
      results.push(result);
      
      if (!result.success) {
        console.error(`❌ Failed to seed ${seeder.collectionName}: ${result.error}`);
        if (this.options.environment === 'production') {
          throw new Error(`Seeding failed for ${seeder.collectionName}`);
        }
      }
      console.log(''); // Add spacing
    }

    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);

    console.log('✅ Seeding Summary:');
    console.log(`   • Successfully seeded: ${successful.length} collections`);
    console.log(`   • Failed: ${failed.length} collections`);
    console.log(`   • Total documents: ${successful.reduce((sum, r) => sum + r.count, 0)}`);
    
    if (failed.length > 0) {
      console.log('\n❌ Failed collections:');
      failed.forEach(r => console.log(`   • ${r.collection}: ${r.error}`));
    }

    return results;
  }

  async runSeeder(collectionName: string): Promise<SeedResult | null> {
    const seeder = this.seeders.find(s => s.collectionName === collectionName);
    if (!seeder) {
      console.error(`❌ No seeder found for collection: ${collectionName}`);
      return null;
    }

    console.log(`📦 Seeding ${collectionName}...`);
    return await seeder.seed();
  }
}
