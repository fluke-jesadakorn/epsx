'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';

interface WindowsPhonePancakeAuthProps {
  onSubmit?: (credentials: { email: string; password: string }) => void;
  variant?: 'user' | 'admin';
  loading?: boolean;
  error?: string;
}

export function WindowsPhonePancakeAuth({
  onSubmit,
  variant = 'user',
  loading = false,
  error
}: WindowsPhonePancakeAuthProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit?.({ email, password });
  };

  const isAdmin = variant === 'admin';

  return (
    <div className={`min-h-screen ${isAdmin ? 'bg-gradient-to-br from-slate-900 via-blue-900 to-slate-800' : 'bg-gradient-to-br from-orange-50 via-yellow-50 to-orange-100'}`}>
      {/* Windows Phone Live Tiles Background Pattern */}
      <div className="fixed inset-0 opacity-10">
        <div className="grid grid-cols-6 gap-2 p-4">
          {Array.from({ length: 30 }).map((_, i) => (
            <motion.div
              key={i}
              className={`h-16 rounded-sm ${isAdmin ? 'bg-blue-400' : 'bg-orange-400'}`}
              animate={{
                scale: [1, 1.02, 1],
                opacity: [0.3, 0.6, 0.3],
              }}
              transition={{
                duration: 3,
                repeat: Infinity,
                delay: i * 0.1,
              }}
            />
          ))}
        </div>
      </div>

      <div className="relative z-10 flex items-center justify-center min-h-screen p-4">
        <motion.div
          initial={{ y: 50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.6 }}
          className="w-full max-w-sm"
        >
          {/* PancakeSwap Logo with Windows Phone Metro Style */}
          <div className="text-center mb-8">
            <motion.div
              className={`w-20 h-20 mx-auto rounded-none ${
                isAdmin 
                  ? 'bg-gradient-to-br from-blue-600 to-blue-800' 
                  : 'bg-gradient-to-br from-orange-400 to-yellow-500'
              } flex items-center justify-center mb-4`}
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
            >
              <div className="text-white text-3xl font-bold">
                {isAdmin ? '⚡' : '🥞'}
              </div>
            </motion.div>
            
            {/* Status indicators - Windows Phone style */}
            <div className="flex justify-center space-x-1 mb-2">
              {Array.from({ length: 4 }).map((_, i) => (
                <motion.div
                  key={i}
                  className={`w-2 h-2 rounded-none ${
                    isAdmin ? 'bg-blue-400' : 'bg-orange-400'
                  }`}
                  animate={{
                    opacity: [0.3, 1, 0.3],
                  }}
                  transition={{
                    duration: 2,
                    repeat: Infinity,
                    delay: i * 0.2,
                  }}
                />
              ))}
            </div>
          </div>

          {/* Error Display - Metro Style */}
          {error && (
            <motion.div
              initial={{ x: -20, opacity: 0 }}
              animate={{ x: 0, opacity: 1 }}
              className="bg-red-600 text-white p-4 mb-4 border-l-4 border-red-800"
            >
              <div className="flex items-center">
                <div className="text-xl mr-3">⚠</div>
                <div className="text-sm font-medium">{error}</div>
              </div>
            </motion.div>
          )}

          {/* Auth Form - Windows Phone Metro Card */}
          <motion.div
            className={`${
              isAdmin 
                ? 'bg-slate-800/90 backdrop-blur-xl border-l-4 border-blue-500' 
                : 'bg-white/90 backdrop-blur-xl border-l-4 border-orange-500'
            } p-6 shadow-2xl`}
            whileHover={{ y: -2 }}
          >
            <form onSubmit={handleSubmit} className="space-y-4">
              {/* Email Input - Metro Style */}
              <div>
                <div className="relative">
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className={`w-full p-4 border-none outline-none font-medium text-lg ${
                      isAdmin 
                        ? 'bg-slate-700 text-white placeholder-slate-400' 
                        : 'bg-gray-50 text-gray-800 placeholder-gray-500'
                    }`}
                    placeholder="📧"
                    required
                  />
                  <div className={`absolute bottom-0 left-0 h-0.5 w-full ${
                    isAdmin ? 'bg-blue-500' : 'bg-orange-500'
                  }`} />
                </div>
              </div>

              {/* Password Input - Metro Style */}
              <div>
                <div className="relative">
                  <input
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className={`w-full p-4 pr-12 border-none outline-none font-medium text-lg ${
                      isAdmin 
                        ? 'bg-slate-700 text-white placeholder-slate-400' 
                        : 'bg-gray-50 text-gray-800 placeholder-gray-500'
                    }`}
                    placeholder="🔐"
                    required
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className={`absolute right-4 top-1/2 transform -translate-y-1/2 text-xl ${
                      isAdmin ? 'text-blue-400' : 'text-orange-500'
                    }`}
                  >
                    {showPassword ? '👁' : '👁‍🗨'}
                  </button>
                  <div className={`absolute bottom-0 left-0 h-0.5 w-full ${
                    isAdmin ? 'bg-blue-500' : 'bg-orange-500'
                  }`} />
                </div>
              </div>

              {/* Submit Button - Windows Phone Style */}
              <motion.button
                type="submit"
                disabled={loading}
                className={`w-full p-4 text-white font-bold text-lg border-none ${
                  isAdmin 
                    ? 'bg-gradient-to-r from-blue-600 to-blue-800' 
                    : 'bg-gradient-to-r from-orange-500 to-yellow-600'
                } ${loading ? 'opacity-50' : ''}`}
                whileHover={{ scale: loading ? 1 : 1.02 }}
                whileTap={{ scale: loading ? 1 : 0.98 }}
              >
                {loading ? (
                  <div className="flex items-center justify-center">
                    <motion.div
                      className="w-6 h-6 border-2 border-white border-t-transparent rounded-full"
                      animate={{ rotate: 360 }}
                      transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
                    />
                  </div>
                ) : (
                  <div className="flex items-center justify-center">
                    <span className="mr-2">{isAdmin ? '🎯' : '🥞'}</span>
                  </div>
                )}
              </motion.button>
            </form>
          </motion.div>

          {/* Status Indicators - Metro Style */}
          <div className="mt-6 flex justify-center space-x-6">
            <motion.div
              className={`flex items-center space-x-2 px-3 py-1 ${
                isAdmin ? 'bg-slate-800/50' : 'bg-white/50'
              } backdrop-blur-sm`}
              animate={{ opacity: [0.7, 1, 0.7] }}
              transition={{ duration: 2, repeat: Infinity }}
            >
              <div className={`w-2 h-2 rounded-full ${isAdmin ? 'bg-green-400' : 'bg-green-500'}`} />
              <span className={`text-xs font-medium ${isAdmin ? 'text-white' : 'text-gray-800'}`}>
                🔒
              </span>
            </motion.div>

            <motion.div
              className={`flex items-center space-x-2 px-3 py-1 ${
                isAdmin ? 'bg-slate-800/50' : 'bg-white/50'
              } backdrop-blur-sm`}
              animate={{ opacity: [0.7, 1, 0.7] }}
              transition={{ duration: 2, repeat: Infinity, delay: 0.5 }}
            >
              <div className={`w-2 h-2 rounded-full ${isAdmin ? 'bg-blue-400' : 'bg-orange-500'}`} />
              <span className={`text-xs font-medium ${isAdmin ? 'text-white' : 'text-gray-800'}`}>
                📊
              </span>
            </motion.div>

            <motion.div
              className={`flex items-center space-x-2 px-3 py-1 ${
                isAdmin ? 'bg-slate-800/50' : 'bg-white/50'
              } backdrop-blur-sm`}
              animate={{ opacity: [0.7, 1, 0.7] }}
              transition={{ duration: 2, repeat: Infinity, delay: 1 }}
            >
              <div className={`w-2 h-2 rounded-full ${isAdmin ? 'bg-yellow-400' : 'bg-yellow-500'}`} />
              <span className={`text-xs font-medium ${isAdmin ? 'text-white' : 'text-gray-800'}`}>
                ⚡
              </span>
            </motion.div>
          </div>

          {/* Progress Bar - Windows Phone Style */}
          <div className={`mt-4 h-1 ${isAdmin ? 'bg-slate-700' : 'bg-gray-200'} overflow-hidden`}>
            <motion.div
              className={`h-full ${isAdmin ? 'bg-blue-500' : 'bg-orange-500'}`}
              animate={{
                x: ['-100%', '100%'],
              }}
              transition={{
                duration: 3,
                repeat: Infinity,
                ease: 'easeInOut',
              }}
            />
          </div>
        </motion.div>
      </div>
    </div>
  );
}