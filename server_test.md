# Server Test

## Request proxied by tollkeeper (no visa)

```sh
curl localhost:8000 --header "Host: wtfismyip.com" --request-target "/json" -o challenge.json
```

## Send payment to tollkeeper

```sh
curl localhost:8080 --request-target "/api/pay" \
  --header "Content-Type: application/json" \ 
  --data @payment.json \
  -o visa.json
```

## Request proxied by tollkeeper (with visa)

```sh
curl localhost:8000 \
  --request-target "/json" \
  --header "Host: wtfismyip.com" \
  --header "X-Keeper-Token: $keeper_token" \
  -o challenge.json
```
