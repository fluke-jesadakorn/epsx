import { AuthClient } from '../clients/AuthClient';
import { PaymentClient } from '../clients/PaymentClient';
import { AnalyticsClient } from '../clients/AnalyticsClient';
import { PermissionsClient } from '../clients/PermissionsClient';

export class ServerApiClient {
  public readonly auth: AuthClient;
  public readonly payments: PaymentClient;
  public readonly analytics: AnalyticsClient;
  public readonly permissions: PermissionsClient;

  constructor(baseUrl?: string) {
    this.auth = new AuthClient(baseUrl);
    this.payments = new PaymentClient(baseUrl);
    this.analytics = new AnalyticsClient(baseUrl);
    this.permissions = new PermissionsClient(baseUrl);
  }
}