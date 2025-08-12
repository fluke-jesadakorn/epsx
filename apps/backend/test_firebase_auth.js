const axios = require('axios');

async function testFirebaseAuth() {
    try {
        // Step 1: Authenticate with Firebase
        console.log('Step 1: Authenticating with Firebase...');
        const firebaseResponse = await axios.post(
            'https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=AIzaSyBof2MIWdFMfpvfl21Di2fOH08ElTgAurU',
            {
                email: 'jesadakorn.kirtnu@gmail.com',
                password: 'Aa_12345678',
                returnSecureToken: true
            }
        );
        
        console.log('Firebase authentication successful!');
        console.log('User UID:', firebaseResponse.data.localId);
        console.log('Email:', firebaseResponse.data.email);
        console.log('ID Token (first 50 chars):', firebaseResponse.data.idToken.substring(0, 50) + '...');
        
        // Step 2: Test backend OIDC token endpoint
        console.log('\nStep 2: Testing backend OIDC token endpoint...');
        const backendResponse = await axios.post(
            'http://localhost:8080/oauth/token',
            new URLSearchParams({
                grant_type: 'authorization_code',
                client_id: 'epsx-frontend',
                code: firebaseResponse.data.idToken
            }),
            {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                }
            }
        );
        
        console.log('Backend authentication successful!');
        console.log('Response:', backendResponse.data);
        
    } catch (error) {
        console.error('Error:', error.response?.data || error.message);
        if (error.response?.status) {
            console.error('Status:', error.response.status);
        }
        if (error.response?.headers) {
            console.error('Headers:', error.response.headers);
        }
    }
}

testFirebaseAuth();