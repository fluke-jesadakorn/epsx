declare module 'firebase-admin' {
  import { App } from 'firebase-admin/app';
  import { Firestore } from 'firebase-admin/firestore';

  interface ServiceAccount {
    type: string;
    project_id: string;
    private_key_id: string;
    private_key: string;
    client_email: string;
    client_id: string;
    auth_uri: string;
    token_uri: string;
    auth_provider_x509_cert_url: string;
    client_x509_cert_url: string;
    universe_domain: string;
  }

  namespace admin {
    function initializeApp(options: { credential: { cert: (serviceAccount: ServiceAccount) => any }; databaseURL: string }): App;
    function app(): App;
    function firestore(app: App): Firestore;
    namespace credential {
      function cert(serviceAccount: ServiceAccount): any;
    }
  }

  export default admin;
}
