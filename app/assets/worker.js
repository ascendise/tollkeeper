async function main(challenge) {
  const stampPrefix = createStampFromChallenge(challenge);
  const stamp = await calcStamp(stampPrefix, challenge.bits);
  postMessage([null, stamp]);
}

function createStampFromChallenge(challenge) {
  const rand = crypto.randomUUID().replace(/-/g, "");
  return `${challenge.ver}:${challenge.bits}:${now()}:${challenge.resource}:${challenge.ext}:${rand}`;
}

// Returns timestamp of current UTC time
function now() {
  const date = new Date();
  const year = padTime(date.getUTCFullYear() % 100);
  const month = padTime(date.getUTCMonth() + 1);
  const day = padTime(date.getUTCDate());
  const hours = padTime(date.getUTCHours());
  const minutes = padTime(date.getUTCMinutes());
  const seconds = padTime(date.getUTCSeconds());
  return `${year}${month}${day}${hours}${minutes}${seconds}`;
}

function padTime(timeElement) {
  return timeElement.toString().padStart(2, "0");
}

async function calcStamp(stamp_prefix, bits) {
  let count = 0;
  let stamp;
  do {
    stamp = `${stamp_prefix}:${count}`;
    postMessage([stamp, null]);
    count++;
  } while (!await isValidStamp(stamp, bits))
  return stamp;
}

async function isValidStamp(stamp, bits) {
  const hash = await crypto.subtle.digest("SHA-1", new TextEncoder().encode(stamp));
  const bytes = new Uint8Array(hash);
  let bitsLeft = bits;
  for (const obyte of bytes) {
    const expectedBits = 8 <= bitsLeft ? 8 : bitsLeft;
    const valid = 0 == (obyte >> (8 - expectedBits));
    if (!valid) {
      return false;
    }
    bitsLeft -= expectedBits;
    if (bitsLeft == 0) {
      return true;
    }
  }
}

onmessage = (e) => {
  const json = e.data[0];
  const challenge = JSON.parse(json);
  main(challenge);
}
