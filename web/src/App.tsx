import React, { useState } from "react";
import { createRoot } from "react-dom/client";

import { AlertTriangle } from "react-feather";

const FILTER = { vendorId: 0xf109, productId: 0xf19d };

const App = () => {
  let hid = window.navigator.hid;
  let [device, setDevice] = useState();
  return (
    <div className="bg-bg text-accent h-screen font-mono flex flex-col items-center justify-center">
      <div className="fixed top-2 left-3 font-bold">
        <span className="text-white">floppapad configurator </span>
        <span>v2.0</span>
      </div>
      {hid ? (
        device ? (
          <div className="flex w-[48rem]">
            <div className="w-1/2 aspect-square border border-2 border-accent mr-2"></div>
            <div className="w-1/2">
              <h1>{device.productName}</h1>
            </div>
          </div>
        ) : (
          <button
            className="font-bold text-2xl p-2 outline outline-accent hover:text-white hover:outline-white hover:bg-fg transition-all flex items-center"
            onClick={() =>
              hid
                .requestDevice({ filters: [FILTER] })
                .then((d) => setDevice(d[0]))
            }
          >
            authorize floppapad
          </button>
        )
      ) : (
        <div className="h-52 flex items-center">
          <AlertTriangle className="w-auto h-full stroke-accent stroke-1" />
          <div>
            <h1 className="font-bold text-2xl">
              your browser doesn't support webhid
            </h1>
            <a
              href="https://caniuse.com/webhid"
              className="text-white underline"
              target="_blank"
            >
              view supported browsers
            </a>
          </div>
        </div>
      )}
    </div>
  );
};

createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
