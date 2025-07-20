import { NextRequest, NextResponse } from 'next/server';

export async function GET(
  _request: NextRequest,
  { params }: { params: { userId: string; featureId: string } }
) {
  try {
    const { featureId } = params;
    
    // Mock feature access check - replace with actual permission evaluation
    // In real implementation, this would check user's package tier, custom permissions, etc.
    const mockFeatureAccess = {
      'dashboard_basic': true,
      'api_personal': true,
      'api_company': true,
      'api_partner': true,
      'data_export': true,
      'beta_features': Math.random() > 0.5, // Random for testing
    };

    const hasAccess = mockFeatureAccess[featureId as keyof typeof mockFeatureAccess] || false;

    return NextResponse.json({ hasAccess });
  } catch (error) {
    console.error('Error checking feature access:', error);
    return NextResponse.json(
      { error: 'Failed to check feature access' },
      { status: 500 }
    );
  }
}
