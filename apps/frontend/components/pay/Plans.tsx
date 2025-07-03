'use client';

const PLANS = [
  { id: 'silver', name: 'Silver', price: 50 },
  { id: 'gold', name: 'Gold', price: 100 },
  { id: 'plat', name: 'Platinum', price: 200 }
];

interface PlansProps {
  val: string;
  set: (pkg: string, amt: number) => void;
}

export function Plans({ val, set }: PlansProps) {
  return (
    <div className="p-6 bg-white rounded-lg shadow-sm">
      <h2 className="text-lg font-semibold mb-4">Choose Your Plan</h2>
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        {PLANS.map(p => (
          <div 
            key={p.id}
            onClick={() => set(p.id, p.price)}
            className={`p-4 border-2 rounded-lg cursor-pointer transition-all ${
              val === p.id 
                ? 'border-blue-500 bg-blue-50' 
                : 'border-gray-200 hover:border-gray-300'
            }`}
          >
            <h3 className="font-bold text-lg">{p.name}</h3>
            <p className="text-2xl font-bold text-blue-600 mt-2">${p.price}</p>
            <p className="text-sm text-gray-500 mt-1">/month</p>
          </div>
        ))}
      </div>
    </div>
  );
}
