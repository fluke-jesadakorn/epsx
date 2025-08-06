import { AnalyticsClient } from '../clients/AnalyticsClient';
import { PaymentClient } from '../clients/PaymentClient';
import { PermissionsClient } from '../clients/PermissionsClient';

export class ClientApiClient {
  public readonly payments: PaymentClient;
  public readonly analytics: AnalyticsClient;
  public readonly permissions: PermissionsClient;

  constructor(baseUrl?: string) {
    this.payments = new PaymentClient(baseUrl);
    this.analytics = new AnalyticsClient(baseUrl);
    this.permissions = new PermissionsClient(baseUrl);
  }
}