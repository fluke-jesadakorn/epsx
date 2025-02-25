declare module "@/configs/header" {
  export const API_HEADERS: {
    'Content-Type': string;
    'Cache-Control': string;
    'Accept': string;
  };

  export const AUTH_HEADERS: (token: string) => {
    'Authorization': string;
    'Content-Type': string;
    'Cache-Control': string;
    'Accept': string;
  };

  export const CORS_HEADERS: {
    'Access-Control-Allow-Origin': string;
    'Access-Control-Allow-Methods': string;
    'Access-Control-Allow-Headers': string;
  };
}
