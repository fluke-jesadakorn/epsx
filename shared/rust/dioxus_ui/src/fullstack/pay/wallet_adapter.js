const {operation, input} = await dioxus.recv();
try {
    const random = () => Array.from(crypto.getRandomValues(new Uint8Array(32)), b => b.toString(16).padStart(2, '0')).join('');
    const stored = key => {let value=sessionStorage.getItem(key);if(!value){value=random();sessionStorage.setItem(key,value);}return value;};
    const provider = () => {const p=window.__epsxPayProvider || window.ethereum; if(!p?.request) throw new Error('No wallet detected. Install or unlock MetaMask, then try again.');return p;};
    let value;
    switch(operation){
      case 'theme': {if(input===null){const saved=localStorage.getItem('theme');value=saved?saved==='dark':matchMedia('(prefers-color-scheme: dark)').matches;}else{localStorage.setItem('theme',input?'dark':'light');value=input;}break;}
      case 'credentials': {
        const cs=input.checkout;
        const token=new URLSearchParams(location.hash.slice(1)).get('token');
        if(cs&&token&&/^[a-f0-9]{64}$/i.test(token))sessionStorage.setItem('epsx.checkout.'+cs,token);
        value={...input,capability:cs?sessionStorage.getItem('epsx.checkout.'+cs):null,guest:stored('epsx.pay.guest')};break;
      }
      case 'key': value=stored('epsx.fullstack.request.'+input);break;
      case 'complete': sessionStorage.removeItem('epsx.fullstack.request.'+input);value=true;break;
      case 'finish_checkout': {const source=sessionStorage.getItem('epsx.fullstack.checkout.source.'+input);if(source){sessionStorage.removeItem('epsx.fullstack.request.'+source);sessionStorage.removeItem('epsx.fullstack.checkout.source.'+input);}value=true;break;}
      case 'copy': await navigator.clipboard.writeText(input);value=true;break;
      case 'remaining': value=Math.max(0,Math.floor((Date.parse(input)-Date.now())/1000));if(!Number.isFinite(value))throw new Error('Checkout expiry unavailable');break;
      case 'pause': await new Promise(resolve=>setTimeout(resolve,4000));value=true;break;
      case 'pairing': value=window.EPSXWalletConnect?.state()?.uri||'';break;
      case 'connect': {
        if(input.walletconnect){
          if(!window.EPSXWalletConnect) throw new Error('WalletConnect is loading. Try again in a moment.');
          const config=await(await fetch('/api/wallet-config',{credentials:'same-origin'})).json();
          if(!config.projectId)throw new Error('WalletConnect is not configured. Use MetaMask or QR transfer.');
          const rpc={56:'https://bsc-dataseed.binance.org',97:'https://data-seed-prebsc-1-s1.binance.org:8545',31337:'http://127.0.0.1:8545'}[input.chain];
          if(!rpc)throw new Error('Unsupported checkout network');
          window.__epsxPayProvider=await window.EPSXWalletConnect.connect({...config,chainId:input.chain,rpcUrl:rpc});
        }else{window.__epsxPayProvider=window.ethereum;}
        const accounts=await provider().request({method:'eth_requestAccounts'});if(!accounts[0])throw new Error('No account selected');value=accounts[0];break;
      }
      case 'disconnect': await window.EPSXWalletConnect?.disconnect();delete window.__epsxPayProvider;value=true;break;
      case 'sign': { const p=provider();const accounts=await p.request({method:'eth_accounts'});if(accounts[0]?.toLowerCase()!==input.address.toLowerCase())throw new Error('Wallet changed. Reconnect and try again.');const message='0x'+Array.from(new TextEncoder().encode(input.message),b=>b.toString(16).padStart(2,'0')).join('');value=await p.request({method:'personal_sign',params:[message,input.address]});break; }
      case 'external_checkout': {const url=new URL(input);if(!['https:','http:'].includes(url.protocol)||url.username||url.password||!url.pathname.startsWith('/checkout/cs_'))throw new Error('Unexpected checkout URL');location.assign(url.href);value=true;break;}
      case 'checkout': {
        const url=new URL(input.url,location.origin);if(url.origin!==location.origin||!/^\/checkout\/(?:cs_[A-Za-z0-9_-]+|[0-9a-f-]{36})$/.test(url.pathname))throw new Error('Unexpected checkout URL');
        sessionStorage.setItem('epsx.fullstack.checkout.source.'+url.pathname.split('/').pop(),input.context);value=url.pathname+url.search+url.hash;break;
      }
      case 'send': {
        const p=provider(), tx=input.transaction;
        const ensure=async()=>{
          const accounts=await p.request({method:'eth_accounts'});if(accounts[0]?.toLowerCase()!==tx.from.toLowerCase())throw new Error('Wallet changed. Reconnect the expected account.');
          let chain=await p.request({method:'eth_chainId'});
          if(BigInt(chain)!==BigInt(tx.chainId)){await p.request({method:'wallet_switchEthereumChain',params:[{chainId:tx.chainId}]});chain=await p.request({method:'eth_chainId'});}
          if(BigInt(chain)!==BigInt(tx.chainId))throw new Error('Select the checkout network in your wallet.');
        };
        await ensure();
        let hash=sessionStorage.getItem(input.storage_key);
        if(hash){const receipt=await p.request({method:'eth_getTransactionReceipt',params:[hash]});if(receipt?.status==='0x0'){sessionStorage.removeItem(input.storage_key);throw new Error('Previous transaction reverted. Retry to prepare a new transaction.');}value=hash;break;}
        if(input.approval){
          let approval=sessionStorage.getItem(input.storage_key+'.approval');
          if(!approval){approval=await p.request({method:'eth_sendTransaction',params:[input.approval]});sessionStorage.setItem(input.storage_key+'.approval',approval);}
          let confirmed=false;
          for(let i=0;i<150;i++){const receipt=await p.request({method:'eth_getTransactionReceipt',params:[approval]});if(receipt){if(receipt.status!=='0x1'){sessionStorage.removeItem(input.storage_key+'.approval');throw new Error('Token approval reverted.');}confirmed=true;break;}await new Promise(r=>setTimeout(r,2000));}
          if(!confirmed)throw new Error('Approval is still pending. Return later to continue.');
        }
        await ensure();hash=await p.request({method:'eth_sendTransaction',params:[tx]});sessionStorage.setItem(input.storage_key,hash);value=hash;break;
      }
      default: throw new Error('Unsupported browser capability');
    }
    dioxus.send({value,error:null});
} catch(error) {dioxus.send({value:null,error:error?.message||String(error)});}
