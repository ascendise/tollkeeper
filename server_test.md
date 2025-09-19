# Request proxied by tollkeeper (no visa)
```sh
curl localhost:9000 --header "Host: wtfismyip.com" --request-target "/json" -o challenge.json
```

# Send payment to tollkeeper
```sh
curl localhost:9100 --request-target "/api/pay" --header "Content-Type: application/json" --data @payment.json -o visa.json
```

# Request proxied by tollkeeper (with visa)
```sh
curl localhost:9000 --header "Host: wtfismyip.com" --header "X-Keeper-Token: $keeper_token" --request-target "/json" -o challenge.json
```
