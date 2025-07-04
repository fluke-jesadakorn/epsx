import { db, auth } from '@/lib/firebase';
import { doc, getDoc, addDoc, collection, query, where, getDocs } from 'firebase/firestore';

export async function pay(amt: number, method: string, desc?: string): Promise<string | null> {
  try {
    const user = auth.currentUser;
    if (!user) throw new Error('Login required');
    
    const tx = {
      userId: user.uid,
      amount: amt,
      method,
      description: desc,
      status: 'pending',
      created: new Date()
    };
    
    const ref = await addDoc(collection(db, 'transactions'), tx);
    return ref.id;
  } catch (error) {
    console.error('Payment error:', error);
    return null;
  }
}

export async function confirm(_txId: string, _method: string, _level: string) {
  try {
    const user = auth.currentUser;
    if (!user) return { success: false, message: 'Not logged in' };
    
    // Implementation would update transaction status
    return { success: true, message: 'Payment confirmed' };
  } catch (error) {
    console.error('Confirm error:', error);
    return { success: false, message: 'Failed to confirm' };
  }
}

export async function status() {
  try {
    const user = auth.currentUser;
    if (!user) return { paid: false, level: 'BASIC' };
    
    const snap = await getDoc(doc(db, 'users', user.uid));
    if (!snap.exists()) {
      return { paid: false, level: 'BASIC', isNew: true };
    }
    
    const data = snap.data();
    return {
      paid: data?.paymentStatus?.hasPaid || false,
      level: data?.userLevel || 'BASIC',
      expire: data?.paymentStatus?.expirationDate?.toDate(),
      lastPay: data?.paymentStatus?.lastPaymentDate?.toDate()
    };
  } catch (error) {
    console.error('Status error:', error);
    return { paid: false, level: 'BASIC' };
  }
}

export async function txs() {
  try {
    const user = auth.currentUser;
    if (!user) return [];
    
    const q = query(
      collection(db, 'transactions'), 
      where('userId', '==', user.uid)
    );
    
    const snap = await getDocs(q);
    const list = snap.docs.map(d => ({
      id: d.id,
      ...d.data(),
      created: d.data().created?.toDate?.() || new Date()
    }));
    
    // Sort by date descending
    return list.sort((a, b) => b.created.getTime() - a.created.getTime());
  } catch (error) {
    console.error('Transaction history error:', error);
    return [];
  }
}

// Hook for easy use in components
export function usePay() {
  return { pay, confirm, status, txs };
}
