import { useEffect } from 'react';
// import { firebaseIAMService } from '../services/firebaseIAMService'; // Service removed
import { PackageTier } from '../types/admin/iam-enhanced';

// Placeholder for removed service
const firebaseIAMService = {
  updateUserPackageTier: async (...args: any[]) => {},
  applyPackagePermissions: async (...args: any[]) => {},
};

export const usePaymentIntegration = () => {
  
  useEffect(() => {
    // Listen for payment success events
    const handlePaymentSuccess = async (event: Event) => {
      const customEvent = event as CustomEvent;
      const { userId, packageTier, transactionId } = customEvent.detail;
      
      try {
        // Apply package permissions automatically
        await firebaseIAMService.applyPackagePermissions(userId, packageTier);
        
        // Log the automatic grant
        console.log(`Auto-applied ${packageTier} permissions for user ${userId} (Transaction: ${transactionId})`);
        
        // Optional: Show notification to admin
        if (window.location.pathname.includes('/admin')) {
          showNotification(`User upgraded to ${packageTier} - Permissions auto-applied`);
        }
        
      } catch (error) {
        console.error('Failed to auto-apply package permissions:', error);
        // Optionally notify admin of the failure
        showNotification(`Warning: Failed to auto-apply permissions for user ${userId}`, 'error');
      }
    };
    
    // Listen for package downgrades/cancellations
    const handlePackageDowngrade = async (event: Event) => {
      const customEvent = event as CustomEvent;
      const { userId, oldTier, newTier } = customEvent.detail;
      
      try {
        // Apply new package permissions (which will replace old ones)
        await firebaseIAMService.updateUserPackageTier(userId, newTier, 'SYSTEM');
        
        console.log(`Downgraded user ${userId} from ${oldTier} to ${newTier}`);
        
      } catch (error) {
        console.error('Failed to handle package downgrade:', error);
      }
    };

    // Listen for subscription expiry
    const handleSubscriptionExpiry = async (event: Event) => {
      const customEvent = event as CustomEvent;
      const { userId } = customEvent.detail;
      
      try {
        // Downgrade to free tier
        await firebaseIAMService.updateUserPackageTier(userId, PackageTier.FREE, 'SYSTEM');
        
        console.log(`User ${userId} subscription expired - downgraded to FREE`);
        
      } catch (error) {
        console.error('Failed to handle subscription expiry:', error);
      }
    };
    
    window.addEventListener('paymentSuccess', handlePaymentSuccess);
    window.addEventListener('packageDowngrade', handlePackageDowngrade);
    window.addEventListener('subscriptionExpiry', handleSubscriptionExpiry);
    
    return () => {
      window.removeEventListener('paymentSuccess', handlePaymentSuccess);
      window.removeEventListener('packageDowngrade', handlePackageDowngrade);
      window.removeEventListener('subscriptionExpiry', handleSubscriptionExpiry);
    };
  }, []);
  
  const showNotification = (message: string, type: 'success' | 'error' = 'success') => {
    // Check if we have a notification system available
    if (typeof window !== 'undefined') {
      // You can integrate with your existing notification system here
      console.log(`[${type.toUpperCase()}] ${message}`);
      
      // Simple browser notification for now
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(message, {
          icon: type === 'success' ? '/icons/success.png' : '/icons/error.png'
        });
      }
    }
  };

  // Manual functions for triggering payment events (for testing/admin use)
  const triggerPaymentSuccess = (userId: string, packageTier: PackageTier, transactionId: string) => {
    const event = new CustomEvent('paymentSuccess', {
      detail: { userId, packageTier, transactionId }
    });
    window.dispatchEvent(event);
  };

  const triggerPackageDowngrade = (userId: string, oldTier: PackageTier, newTier: PackageTier) => {
    const event = new CustomEvent('packageDowngrade', {
      detail: { userId, oldTier, newTier }
    });
    window.dispatchEvent(event);
  };

  const triggerSubscriptionExpiry = (userId: string) => {
    const event = new CustomEvent('subscriptionExpiry', {
      detail: { userId }
    });
    window.dispatchEvent(event);
  };

  return {
    triggerPaymentSuccess,
    triggerPackageDowngrade,
    triggerSubscriptionExpiry,
  };
};
