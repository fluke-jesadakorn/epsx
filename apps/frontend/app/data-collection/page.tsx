'use client';

import React, { useState } from 'react';

export default function DataCollectionHub() {
  const [dataPoints, setDataPoints] = useState([
    { id: 1, identifier: 'ITEM-A', quantity: 10, refValue: 50, date: '2023-01-15' },
    { id: 2, identifier: 'ITEM-B', quantity: 5, refValue: 75, date: '2023-02-20' },
    { id: 3, identifier: 'ITEM-C', quantity: 8, refValue: 30, date: '2023-03-10' },
  ]);

  const [newData, setNewData] = useState({ identifier: '', quantity: '', refValue: '', date: '' });

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setNewData(prev => ({ ...prev, [name]: value }));
  };

  const handleAddData = (e: React.FormEvent) => {
    e.preventDefault();
    if (newData.identifier && newData.quantity && newData.refValue && newData.date) {
      setDataPoints(prev => [
        ...prev,
        {
          id: prev.length + 1,
          identifier: newData.identifier,
          quantity: parseInt(newData.quantity),
          refValue: parseInt(newData.refValue),
          date: newData.date,
        },
      ]);
      setNewData({ identifier: '', quantity: '', refValue: '', date: '' });
      alert('Data point added successfully!');
    } else {
      alert('Please fill in all fields.');
    }
  };

  const handleDeleteData = (id: number) => {
    setDataPoints(prev => prev.filter(item => item.id !== id));
    alert('Data point deleted.');
  };

  return (
    <div className="container mx-auto p-4 sm:p-6 bg-gradient-to-r from-blue-50 to-indigo-50 min-h-screen">
      <div className="mb-6 sm:mb-8 animate-fade-in">
        <h1 className="text-2xl sm:text-3xl font-extrabold mb-3 text-indigo-700">Data Collection Hub</h1>
        <p className="text-gray-600 text-base sm:text-lg">Manage and analyze your data sets for comparison and insights.</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Data Input Form */}
        <div className="p-4 sm:p-8 bg-white border rounded-2xl shadow-lg transform transition-all hover:shadow-xl">
          <h2 className="text-xl sm:text-2xl font-bold mb-4 sm:mb-6 text-gray-800">Add New Data Point</h2>
          <form onSubmit={handleAddData} className="space-y-4 sm:space-y-5">
            <div className="relative">
              <label className="block text-sm font-medium mb-2 text-gray-700">Item Identifier</label>
              <input
                type="text"
                name="identifier"
                value={newData.identifier}
                onChange={handleInputChange}
                className="w-full p-2 sm:p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
                placeholder="e.g., ITEM-X"
              />
            </div>
            <div className="relative">
              <label className="block text-sm font-medium mb-2 text-gray-700">Quantity</label>
              <input
                type="number"
                name="quantity"
                value={newData.quantity}
                onChange={handleInputChange}
                className="w-full p-2 sm:p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
                placeholder="e.g., 10"
              />
            </div>
            <div className="relative">
              <label className="block text-sm font-medium mb-2 text-gray-700">Reference Value</label>
              <input
                type="number"
                name="refValue"
                value={newData.refValue}
                onChange={handleInputChange}
                className="w-full p-2 sm:p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
                placeholder="e.g., 50"
              />
            </div>
            <div className="relative">
              <label className="block text-sm font-medium mb-2 text-gray-700">Acquisition Date</label>
              <input
                type="date"
                name="date"
                value={newData.date}
                onChange={handleInputChange}
                className="w-full p-2 sm:p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
              />
            </div>
            <button type="submit" className="w-full bg-indigo-600 text-white py-2 sm:py-3 rounded-lg hover:bg-indigo-700 transition-colors duration-200 font-semibold">
              Add Data Point
            </button>
          </form>
        </div>

        {/* Data List */}
        <div className="p-4 sm:p-8 bg-white border rounded-2xl shadow-lg md:col-span-2 transform transition-all hover:shadow-xl">
          <h2 className="text-xl sm:text-2xl font-bold mb-4 sm:mb-6 text-gray-800">Your Data Set</h2>
          {dataPoints.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="w-full text-left rounded-lg">
                <thead>
                  <tr className="bg-gray-100 border-b border-gray-200">
                    <th className="p-2 sm:p-4 font-semibold text-gray-700">Identifier</th>
                    <th className="p-2 sm:p-4 font-semibold text-gray-700">Quantity</th>
                    <th className="p-2 sm:p-4 font-semibold text-gray-700">Ref. Value</th>
                    <th className="p-2 sm:p-4 font-semibold text-gray-700">Date</th>
                    <th className="p-2 sm:p-4 font-semibold text-gray-700">Action</th>
                  </tr>
                </thead>
                <tbody>
                  {dataPoints.map(point => (
                    <tr key={point.id} className="border-b border-gray-200 hover:bg-gray-50 transition-colors">
                      <td className="p-2 sm:p-4 text-gray-800 font-medium">{point.identifier}</td>
                      <td className="p-2 sm:p-4 text-gray-600">{point.quantity}</td>
                      <td className="p-2 sm:p-4 text-gray-600">{point.refValue}</td>
                      <td className="p-2 sm:p-4 text-gray-600">{point.date}</td>
                      <td className="p-2 sm:p-4">
                        <button
                          onClick={() => handleDeleteData(point.id)}
                          className="text-red-500 hover:text-red-700 font-medium transition-colors"
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <p className="text-gray-500 text-center py-3 sm:py-4">No data points added yet.</p>
          )}
        </div>

        {/* Analysis Section */}
        <div className="p-4 sm:p-8 bg-white border rounded-2xl shadow-lg md:col-span-2 transform transition-all hover:shadow-xl">
          <h2 className="text-xl sm:text-2xl font-bold mb-4 sm:mb-6 text-gray-800">Data Analysis</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-6">
            <div className="p-3 sm:p-6 bg-blue-50 border border-blue-100 rounded-xl shadow-sm hover:shadow-md transition-shadow">
              <h3 className="font-semibold mb-2 sm:mb-3 text-blue-800">Total Metrics</h3>
              <p className="text-gray-700 text-base sm:text-lg font-medium">Combined Value: 980</p>
            </div>
            <div className="p-3 sm:p-6 bg-green-50 border border-green-100 rounded-xl shadow-sm hover:shadow-md transition-shadow">
              <h3 className="font-semibold mb-2 sm:mb-3 text-green-800">Trend Comparison</h3>
              <p className="text-gray-700 text-base sm:text-lg font-medium">Your Data vs. Standard: +3.2% Variation</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
