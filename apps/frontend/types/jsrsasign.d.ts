declare module 'jsrsasign' {
  export function hex2b64(hex: string): string;
  
  export namespace KJUR {
    namespace crypto {
      class Signature {
        constructor(options: {
          alg: string;
          prov: string;
          prvkeypem?: string;
          pubkeypem?: string;
        });
        
        updateString(str: string): void;
        sign(): string;
        verify(signature: string): boolean;
      }
    }
  }
}
