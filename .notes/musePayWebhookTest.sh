#!/bin/bash

# This script sends a test webhook request to the MusePay endpoint
# Note: The signature field is a placeholder and should be replaced with a valid signature if verification is required.
# For testing purposes, you might need to temporarily bypass signature verification in the code.

curl -X POST \
  http://localhost:3000/api/webhook/musepay \
  -H "Content-Type: application/json" \
  -d '{
    "actual_amount": "9.870300000000000000",
    "currency": "USDT_BSC",
    "customer_ref_id": "PARTNER:ADDRESS:MAIN_ACCOUNT",
    "extra_info": "{\"blockHeight\":\"52608196\",\"channel\":\"fireblocks_api\",\"customerRefId\":\"PARTNER:ADDRESS:MAIN_ACCOUNT\",\"description\":\"private_address\",\"destinationAddress\":\"0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59\",\"network\":\"BNB\",\"networkCurrency\":\"BNB_BSC\",\"networkFee\":\"0\",\"networkTxnCurrency\":\"USDT_BSC\",\"numOfConfirms\":\"1\",\"payType\":\"on_chain\",\"payeeUserType\":\"3\",\"payerUserType\":\"3\",\"riskId\":\"17de016a-1513-4ec4-8401-46a30b47dfbc\",\"riskLevel\":\"PASS\",\"sourceAddress\":\"0x26d0f8Ee0F1a47d42f87549EC10f59b796Bc9d60\",\"txnHash\":\"0xffdea633a4ff695f34d5fa470a8351f0b503d94352adc2a755e160ead38c1ac1\",\"userInfo\":{}}",
    "fee_amount": "0.029700000000000000",
    "finish_time": "1751453960000",
    "order_amount": "9.900000000000000000",
    "order_no": "2025070235000370198105919593",
    "order_type": "charge",
    "partner_id": "20001911",
    "pay_amount": "9.900000000000000000",
    "product_code": "m_charge",
    "reason": "",
    "request_id": "2025070209089440360483611456",
    "settle_currency": "USDT_BSC",
    "sign": "placeholder_signature_for_testing",
    "status": 99
  }'

echo "Test webhook request sent to MusePay endpoint."
