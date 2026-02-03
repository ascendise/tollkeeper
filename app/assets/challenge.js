async function main() {
  const tollHalJson = document.getElementById('toll').innerHTML;
  const tollHal = JSON.parse(tollHalJson);
  const worker = await startWorker();
  worker.postMessage([JSON.stringify(tollHal.toll.challenge)])
  const currentStamp = document.getElementById('stamp');
  worker.onmessage = (e) => {
    if (e.data[0]) {
      currentStamp.innerHTML = e.data[0];
    } else {
      payToll(tollHal, e.data[1])
    }
  }
}

async function startWorker() {
  const workerUrl = document.getElementById('workerScript').href;
  const workerRes = await fetch(workerUrl);
  const workerJs = await workerRes.text();
  const workerBlob = new Blob([workerJs], {
    type: "text/javascript"
  });
  return new Worker(URL.createObjectURL(workerBlob));
}

async function payToll(tollHal, stamp) {
  const payment = {
    toll: tollHal.toll,
    value: stamp
  };
  const payLink = tollHal._links.pay;
  const response = await fetch(payLink, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payment)
  });
  const visa = await response.json();
  document.cookie = `${visa.header_name}=${visa.token}; path=/`;
  window.location.href = `http://${visa._links.origin_url}`;

}

main();
