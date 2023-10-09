import React from "react";
import { createRoot } from "react-dom/client";


const Popup = () => {
    const [data, setData] = React.useState<string>();

    React.useEffect(() => {
      chrome.runtime.sendMessage({type: "GetAllWorkspaces"}, response => {
        console.log("Got Response", response);
        setData(JSON.stringify(response))
      });
    }, []);

  return (
    <div className="App">
        <button> Click me </button>
        <p> Data: {data} </p>
    </div>
  );
};


function init() {
  const appContainer = document.querySelector("#app-container");
  if (!appContainer) {
    throw new Error("Can not find #app-container");
  }
  const root = createRoot(appContainer);
  root.render(<Popup />);
}

init();

export default Popup;

