async function isValidStamp(stamp, bits) {
  let hash = await window.crypto.subtle.digest("SHA-1", new TextEncoder().encode(stamp));
  bytes = new Uint8Array(hash);
  let bits_left = bits;
  console.log(stamp);
  for (const obyte of bytes) {
    let expected_bits = 8 <= bits_left ? 8 : bits_left;
    console.log(`expected: ${expected_bits}`)
    let valid = 0 == (obyte >> (8 - expected_bits));
    console.log(`valid: ${valid}`)
    if(!valid) {
      return false;
    }
    bits_left -= expected_bits;
    if(bits_left == 0) {
      console.log(`Valid stamp found: ${stamp}`)
      return true;
    }
  }
}

async function calcStamp(stamp_prefix, bits) {
  let stamp_el = document.getElementById('stamp');
  let count = 0;
  let stamp;
  do {
    stamp = `${stamp_prefix}:${count}`
    stamp_el.innerHTML = stamp;
    count++;
  } while (!await isValidStamp(stamp, bits))
  return stamp;
}

function padTime(timeElement) {
  return timeElement.toString().padStart(2, "0");
}

async function main() {
  const toll_json = document.getElementById('toll').innerHTML;
  const toll_res = JSON.parse(toll_json);
  const date = new Date();
  const year = padTime(date.getUTCFullYear() % 100);
  const month = padTime(date.getUTCMonth() + 1);
  const day = padTime(date.getUTCDate());
  const hours = padTime(date.getUTCHours());
  const minutes = padTime(date.getUTCMinutes());
  const seconds = padTime(date.getUTCSeconds());
  const date_string = `${year}${month}${day}${hours}${minutes}${seconds}`;
  const rand = window.crypto.randomUUID().replace(/-/g, "");
  const toll = toll_res.toll;
  const challenge = toll.challenge;
  const stamp_prefix = `${challenge.ver}:${challenge.bits}:${date_string}:${challenge.resource}:${challenge.ext}:${rand}`;
  const stamp = await calcStamp(stamp_prefix, challenge.bits)
  const payment = {
    toll: toll,
    value: stamp
  };
  const pay_link = toll_res._links.pay;
  const response = await fetch(pay_link, {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify(payment)
  });
  let visa = await response.json();
  document.cookie = `${visa.header_name}=${visa.token}; path=/`;
  window.location.href = `http://${visa._links.origin_url}`;
}

main();
