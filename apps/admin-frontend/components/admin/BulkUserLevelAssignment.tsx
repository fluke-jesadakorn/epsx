'use client';

import { useState } from 'react';
import { AdminApiService } from '@/services/adminApiService';
import { UserLevel, USER_LEVEL_CONFIGS } from '@/types/admin/userLevels';
import { Upload, Users } from 'lucide-react';
import { useToast } from '@/components/ui/toast';

export function BulkUserLevelAssignment() {
  const [isLoading, setIsLoading] = useState(false);
  const [selectedLevel, setSelectedLevel] = useState<UserLevel>(UserLevel.BRONZE);
  const [userEmails, setUserEmails] = useState('');
  const [reason, setReason] = useState('');
  const { addToast } = useToast();

  const handleBulkAssignment = async () => {
    if (!userEmails.trim()) {
      addToast({
        type: 'warning',
        title: 'Please enter user emails'
      });
      return;
    }

    const emails = userEmails.split('\n').filter(email => email.trim());
    
    if (emails.length === 0) {
      addToast({
        type: 'warning',
        title: 'Please enter valid user emails'
      });
      return;
    }

    try {
      setIsLoading(true);
      
      // Filter out empty lines
      const identifiers = userEmails.split('\n').filter(identifier => identifier.trim());
      
      if (identifiers.length === 0) {
        addToast({
          type: 'warning',
          title: 'Please enter valid user emails or UIDs'
        });
        return;
      }

      const updates = identifiers.map(identifier => ({
        uid: identifier.trim(),
        level: selectedLevel,
        reason: reason || 'Bulk assignment'
      }));

      const response = await AdminApiService.bulkUpdateLevels(updates);
      
      
      // Check if response has results property
      if (response && response.results && Array.isArray(response.results)) {
        // Show detailed results
        const successCount = response.results.filter((r: any) => r.success).length;
        const failureCount = response.results.filter((r: any) => !r.success).length;
        
        if (failureCount > 0) {
          const failedUsers = response.results.filter((r: any) => !r.success).map((r: any) => r.uid).join(', ');
          addToast({
            type: 'warning',
            title: `Updated ${successCount} users successfully`,
            description: `Failed to update ${failureCount} users: ${failedUsers}`
          });
        } else {
          addToast({
            type: 'success',
            title: `Successfully updated ${successCount} users`,
            description: `All users upgraded to ${USER_LEVEL_CONFIGS[selectedLevel].name} level`
          });
        }
      } else {
        // Fallback if response doesn't have results
        console.warn('Response does not have results property:', response);
        addToast({
          type: 'success',
          title: 'Bulk assignment completed',
          description: `Updated ${identifiers.length} users to ${USER_LEVEL_CONFIGS[selectedLevel].name} level`
        });
      }
      
      setUserEmails('');
      setReason('');
    } catch (error: any) {
      console.error('Bulk assignment failed:', error);
      
      // Check if the error has more specific information
      if (error.message && error.message.includes('Internal server error')) {
        addToast({
          type: 'error',
          title: 'Bulk assignment failed',
          description: 'Server error occurred. Please check the server logs for more details.'
        });
      } else {
        addToast({
          type: 'error',
          title: 'Bulk assignment failed',
          description: error.message
        });
      }
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center gap-2 mb-4">
        <Users className="h-5 w-5 text-blue-600" />
        <h3 className="text-lg font-semibold">Bulk User Level Assignment</h3>
      </div>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Select Level</label>
          <select
            value={selectedLevel}
            onChange={(e) => setSelectedLevel(e.target.value as UserLevel)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          >
            {Object.values(UserLevel).map((level) => {
              const config = USER_LEVEL_CONFIGS[level];
              return (
                <option key={level} value={level}>
                  {config.name} - {config.description}
                </option>
              );
            })}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">User Emails or UIDs (one per line)</label>
          <textarea
            value={userEmails}
            onChange={(e) => setUserEmails(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            rows={6}
            placeholder="user1@example.com&#10;user2@example.com&#10;Or use UIDs:&#10;abc123def456&#10;xyz789ghi012"
          />
          <p className="text-xs text-gray-500 mt-1">
            You can enter either email addresses or Firebase UIDs. The system will automatically detect which format you&apos;re using.
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Reason</label>
          <input
            type="text"
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            placeholder="Enter reason for bulk assignment..."
          />
        </div>

        <div className="flex justify-end gap-3">
          <button
            onClick={handleBulkAssignment}
            disabled={isLoading}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            <Upload className="h-4 w-4" />
            {isLoading ? 'Assigning...' : 'Bulk Assign'}
          </button>
        </div>
      </div>
    </div>
  );
}
