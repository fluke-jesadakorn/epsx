/**
 * Marketplace Catalog Component
 * Enterprise product discovery and browsing interface
 */
'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { useEnterpriseApi, MarketplaceProduct, IntegrationProduct, ProfessionalService } from '@/lib/enterprise-api-client';
import { useWeb3AuthenticatedState } from '@/lib/auth/web3-store';

interface MarketplaceCatalogProps {
  initialCategory?: string;
}

export function MarketplaceCatalog({ initialCategory = 'all' }: MarketplaceCatalogProps) {
  const enterpriseApi = useEnterpriseApi();
  const { enterpriseTier, hasApiAccess } = useWeb3AuthenticatedState();

  // State management
  const [products, setProducts] = useState<MarketplaceProduct[]>([]);
  const [integrations, setIntegrations] = useState<IntegrationProduct[]>([]);
  const [services, setServices] = useState<ProfessionalService[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeCategory, setActiveCategory] = useState(initialCategory);
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'price' | 'popularity' | 'rating'>('popularity');
  const [filterTier, setFilterTier] = useState<string>('all');

  // Load marketplace data
  const loadMarketplaceData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      if (activeCategory === 'all' || activeCategory === 'products') {
        const productsResponse = await enterpriseApi.getMarketplaceCatalog({
          search: searchQuery || undefined,
          tier: filterTier !== 'all' ? filterTier : undefined,
          page: 1,
          per_page: 50,
        });

        if (productsResponse.success && productsResponse.data) {
          setProducts(productsResponse.data.items);
        } else {
          console.warn('Failed to load products:', productsResponse.error);
        }
      }

      if (activeCategory === 'all' || activeCategory === 'integrations') {
        const integrationsResponse = await enterpriseApi.getIntegrations();
        
        if (integrationsResponse.success && integrationsResponse.data) {
          setIntegrations(integrationsResponse.data);
        } else {
          console.warn('Failed to load integrations:', integrationsResponse.error);
        }
      }

      if (activeCategory === 'all' || activeCategory === 'services') {
        const servicesResponse = await enterpriseApi.getProfessionalServices();
        
        if (servicesResponse.success && servicesResponse.data) {
          setServices(servicesResponse.data);
        } else {
          console.warn('Failed to load services:', servicesResponse.error);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load marketplace data');
    } finally {
      setLoading(false);
    }
  }, [activeCategory, searchQuery, filterTier, enterpriseApi]);

  useEffect(() => {
    loadMarketplaceData();
  }, [loadMarketplaceData]);

  // Add to cart functionality
  const addToCart = async (productId: string, productType: 'product' | 'integration' | 'service') => {
    try {
      const response = await enterpriseApi.addToCart({
        product_id: productId,
        product_type: productType,
        quantity: 1,
      });

      if (response.success) {
        // Show success notification
        console.log('Added to cart successfully');
      } else {
        console.error('Failed to add to cart:', response.error);
      }
    } catch (err) {
      console.error('Error adding to cart:', err);
    }
  };

  // Check if user can access tier
  const canAccessTier = (requiredTier: string) => {
    const tierHierarchy = { 'Starter': 1, 'Business': 2, 'Enterprise': 3, 'Whale': 4 };
    const userLevel = tierHierarchy[enterpriseTier as keyof typeof tierHierarchy] || 0;
    const requiredLevel = tierHierarchy[requiredTier as keyof typeof tierHierarchy] || 1;
    return userLevel >= requiredLevel;
  };

  // Render product card
  const renderProductCard = (product: MarketplaceProduct) => (
    <div key={product.id} className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{product.name}</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400">{product.category}</p>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold text-green-600">${product.price_usd}</div>
          <div className="text-sm text-gray-500">
            {product.billing_type === 'monthly' ? '/month' : 
             product.billing_type === 'annual' ? '/year' : 'one-time'}
          </div>
        </div>
      </div>

      <p className="text-gray-700 dark:text-gray-300 mb-4 line-clamp-3">{product.description}</p>

      <div className="mb-4">
        <div className="flex items-center gap-2 mb-2">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Required Tier:</span>
          <span className={`px-2 py-1 rounded text-xs font-medium ${
            canAccessTier(product.tier_requirement) 
              ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300'
              : 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300'
          }`}>
            {product.tier_requirement}
          </span>
        </div>
        
        {product.features.length > 0 && (
          <div className="space-y-1">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Features:</span>
            <ul className="text-sm text-gray-600 dark:text-gray-400 space-y-1">
              {product.features.slice(0, 3).map((feature, idx) => (
                <li key={idx} className="flex items-center gap-2">
                  <span className="text-green-500">✓</span>
                  {feature}
                </li>
              ))}
              {product.features.length > 3 && (
                <li className="text-gray-500">+{product.features.length - 3} more features</li>
              )}
            </ul>
          </div>
        )}
      </div>

      <div className="flex gap-2">
        <button 
          onClick={() => addToCart(product.id, 'product')}
          disabled={!canAccessTier(product.tier_requirement)}
          className="flex-1 bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
        >
          {canAccessTier(product.tier_requirement) ? 'Add to Cart' : 'Tier Required'}
        </button>
        <button className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
          Details
        </button>
      </div>
    </div>
  );

  // Render integration card
  const renderIntegrationCard = (integration: IntegrationProduct) => (
    <div key={integration.id} className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{integration.name}</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400">by {integration.provider}</p>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold text-blue-600">${integration.price_usd}</div>
          <div className="text-sm text-gray-500">setup fee</div>
        </div>
      </div>

      <p className="text-gray-700 dark:text-gray-300 mb-4 line-clamp-3">{integration.description}</p>

      <div className="grid grid-cols-2 gap-4 mb-4 text-sm">
        <div>
          <span className="font-medium text-gray-700 dark:text-gray-300">Complexity:</span>
          <span className={`ml-2 px-2 py-1 rounded text-xs ${
            integration.setup_complexity === 'easy' ? 'bg-green-100 text-green-800' :
            integration.setup_complexity === 'medium' ? 'bg-yellow-100 text-yellow-800' :
            'bg-red-100 text-red-800'
          }`}>
            {integration.setup_complexity}
          </span>
        </div>
        <div>
          <span className="font-medium text-gray-700 dark:text-gray-300">Time:</span>
          <span className="ml-2">{integration.integration_time_hours}h</span>
        </div>
      </div>

      <div className="mb-4">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Networks:</span>
        <div className="flex flex-wrap gap-1 mt-1">
          {integration.supported_networks.map((network, idx) => (
            <span key={idx} className="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-xs rounded">
              {network}
            </span>
          ))}
        </div>
      </div>

      <div className="flex gap-2">
        <button 
          onClick={() => addToCart(integration.id, 'integration')}
          className="flex-1 bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 transition-colors"
        >
          Add to Cart
        </button>
        <button className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
          Details
        </button>
      </div>
    </div>
  );

  // Render service card
  const renderServiceCard = (service: ProfessionalService) => (
    <div key={service.id} className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{service.name}</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400">{service.service_type}</p>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold text-purple-600">${service.price_usd.toLocaleString()}</div>
          <div className="text-sm text-gray-500">{service.duration_weeks} weeks</div>
        </div>
      </div>

      <p className="text-gray-700 dark:text-gray-300 mb-4 line-clamp-3">{service.description}</p>

      <div className="mb-4">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Expertise Required:</span>
        <div className="flex flex-wrap gap-1 mt-1">
          {service.expertise_required.map((skill, idx) => (
            <span key={idx} className="px-2 py-1 bg-purple-100 dark:bg-purple-900/20 text-purple-800 dark:text-purple-300 text-xs rounded">
              {skill}
            </span>
          ))}
        </div>
      </div>

      <div className="mb-4">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Deliverables:</span>
        <ul className="text-sm text-gray-600 dark:text-gray-400 mt-1 space-y-1">
          {service.deliverables.slice(0, 2).map((deliverable, idx) => (
            <li key={idx} className="flex items-center gap-2">
              <span className="text-purple-500">•</span>
              {deliverable}
            </li>
          ))}
          {service.deliverables.length > 2 && (
            <li className="text-gray-500">+{service.deliverables.length - 2} more deliverables</li>
          )}
        </ul>
      </div>

      <div className="flex gap-2">
        <button 
          onClick={() => addToCart(service.id, 'service')}
          className="flex-1 bg-purple-600 text-white py-2 px-4 rounded-lg hover:bg-purple-700 transition-colors"
        >
          Request Quote
        </button>
        <button className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
          Details
        </button>
      </div>
    </div>
  );

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600 dark:text-gray-400">Loading marketplace...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <div className="flex items-center">
          <span className="text-red-600 dark:text-red-400 mr-2">⚠️</span>
          <span className="text-red-800 dark:text-red-300">Failed to load marketplace: {error}</span>
        </div>
        <button 
          onClick={loadMarketplaceData}
          className="mt-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Enterprise Marketplace</h1>
          <p className="text-gray-600 dark:text-gray-400">
            Your enterprise tier: <span className="font-medium text-blue-600">{enterpriseTier}</span>
            {hasApiAccess && <span className="ml-2 text-green-600">• API Access Enabled</span>}
          </p>
        </div>
        <div className="flex gap-2">
          <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
            View Cart
          </button>
          <button className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            My Subscriptions
          </button>
        </div>
      </div>

      {/* Filters and Search */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Search
            </label>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search products, integrations..."
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Category
            </label>
            <select
              value={activeCategory}
              onChange={(e) => setActiveCategory(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value="all">All Categories</option>
              <option value="products">Products</option>
              <option value="integrations">Integrations</option>
              <option value="services">Professional Services</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Enterprise Tier
            </label>
            <select
              value={filterTier}
              onChange={(e) => setFilterTier(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value="all">All Tiers</option>
              <option value="Starter">Starter</option>
              <option value="Business">Business</option>
              <option value="Enterprise">Enterprise</option>
              <option value="Whale">Whale</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Sort By
            </label>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value="popularity">Popularity</option>
              <option value="price">Price</option>
              <option value="rating">Rating</option>
            </select>
          </div>
        </div>
      </div>

      {/* Products Grid */}
      {(activeCategory === 'all' || activeCategory === 'products') && products.length > 0 && (
        <div>
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">Products</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {products.map(renderProductCard)}
          </div>
        </div>
      )}

      {/* Integrations Grid */}
      {(activeCategory === 'all' || activeCategory === 'integrations') && integrations.length > 0 && (
        <div>
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">Integrations</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {integrations.map(renderIntegrationCard)}
          </div>
        </div>
      )}

      {/* Services Grid */}
      {(activeCategory === 'all' || activeCategory === 'services') && services.length > 0 && (
        <div>
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">Professional Services</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {services.map(renderServiceCard)}
          </div>
        </div>
      )}

      {/* Empty State */}
      {products.length === 0 && integrations.length === 0 && services.length === 0 && (
        <div className="text-center py-12">
          <div className="text-gray-400 mb-4">🏪</div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No items found</h3>
          <p className="text-gray-600 dark:text-gray-400">
            Try adjusting your search criteria or check back later for new products.
          </p>
        </div>
      )}
    </div>
  );
}