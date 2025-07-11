'use client';

const PAYS = [
  { id: 'USDT_TRC20', name: 'USDT TRC20', fee: '$0.1', fast: true },
  { id: 'USDT_BSC', name: 'USDT BSC', fee: '$0.2' },
  { id: 'USDT_ERC20', name: 'USDT ERC20', fee: '$2-15' },
  // Temporarily disabled as per request
  // { id: 'card', name: 'Credit Card', fee: '2.9%' }
];

interface MethodsProps {
  val: string;
  set: (pay: string) => void;
}

export function Methods({ val, set }: MethodsProps) {
  return (
    <div className="p-6 bg-white rounded-lg shadow-sm">
      <h2 className="text-lg font-semibold mb-4">Payment Method</h2>
      <div className="space-y-2">
        {PAYS.map((m) => (
          <label
            key={m.id}
            className="flex items-center p-3 border rounded-lg cursor-pointer hover:bg-gray-50 transition-colors"
          >
            <input
              type="radio"
              name="payment"
              checked={val === m.id}
              onChange={() => set(m.id)}
              className="mr-3"
            />
            <span className="flex-1 font-medium">{m.name}</span>
            {m.fast && (
              <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded mr-3">
                Fastest
              </span>
            )}
            <span className="text-sm text-gray-500">Fee: {m.fee}</span>
          </label>
        ))}
      </div>
    </div>
  );
}
