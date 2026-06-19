import init, { new_datapackage_store, new_session } from "./pkg/client.js";

const form = document.getElementById("conn-info");
const server = document.getElementById("server");
const port = document.getElementById("port");
const slot = document.getElementById("slot");
const password = document.getElementById("password");
const msgBuffer = document.getElementById("msg-buffer");
const clientElem = document.getElementById("client");
const clear = document.getElementsByClassName("clear");

server.value = localStorage.getItem("server") ?? "archipelago.gg";
port.value = localStorage.getItem("port") ?? "38281";
slot.value = localStorage.getItem("slot") ?? "Player";

form.addEventListener("submit", (e) => {
  e.preventDefault();

  localStorage.setItem("server", server.value);
  localStorage.setItem("port", port.value);
  localStorage.setItem("slot", slot.value);

  tryConnect(e);
});

async function disconnect(e) {
  e.stopPropagation();
  isClosing = true;
  if (client) client.close();
}

document.getElementById("disconnect").addEventListener("click", disconnect);

clientElem.addEventListener("click", (e) => {
  if (e.target.id) {
    const action = session.handle_click(e.target.id);
    const locations = Array.from(action.locations()).map(Number);

    if (locations.length > 0) {
      client.send(
        JSON.stringify([
          {
            cmd: "LocationChecks",
            locations: locations,
          },
        ]),
      );
    }

    if (action.victory) {
      client.send(JSON.stringify([{ cmd: "StatusUpdate", status: 30 }]));
    }
  }
});

let tryConnect = (e) => console.log("Please wait for wasm to initialise.");
let printJson = (data) => console.log(data);

let client;
let roomInfo;
let datapackageStore;
let session;
let receivedItemIndex = 0;
let isClosing = false;

async function run() {
  await init();

  tryConnect = async () => {
    form.hidden = true;
    try {
      await initialConnect(true);
    } catch {
      try {
        await initialConnect(false);
      } catch {
        form.hidden = false;
      }
    }

    client.onmessage = handleEvent;
    client.onclose = handleClose;
  };
}

run();

function initialConnect(secure) {
  return new Promise((resolve, reject) => {
    client = new WebSocket(`${secure ? "wss" : "ws"}://${server.value}:${port.value}`);

    client.onopen = () => {
      if (client) {
        resolve();
      } else {
        reject(["Socket was closed unexpectedly."]);
      }
    };

    client.onerror = (event) => {
      reject([event]);
    };
  });
}

async function roomInfoConnect() {
  datapackageStore = new_datapackage_store(JSON.stringify(roomInfo));
  await datapackageStore.get_fs();
  await datapackageStore.load_cached_datapackages();
  const missing = datapackageStore.get_missing_games();
  if (missing.length > 0) {
    console.log(`Missing datapackages for games ${missing}`);
    client.send(JSON.stringify([{ cmd: "GetDataPackage", games: missing }]));
  } else {
    datapackageConnect();
  }
}

async function datapackageConnect(datapackage) {
  if (datapackage) {
    for (const [game, data] of Object.entries(datapackage.data.games)) {
      await datapackageStore.add_game(game, JSON.stringify(data));
    }
  }
  if (!localStorage.getItem("apAhlcgUuid")) localStorage.setItem("apAhlcgUuid", Math.random() * (1 << 16));
  client.send(
    JSON.stringify([
      {
        cmd: "Connect",
        password: password.value,
        game: "Arkham Horror The Card Game",
        name: slot.value,
        uuid: localStorage.getItem("apAhlcgUuid"),
        version: { major: 0, minor: 6, build: 2, class: "Version" },
        items_handling: 7,
        tags: [],
        slot_data: true,
      },
    ]),
  );
}

function connectedConnect(connected) {
  session = new_session(datapackageStore, JSON.stringify(connected), slot.value);
  printJson = (data) => {
    const newMsg = document.createElement("li");
    newMsg.innerHTML = session.try_format_json(JSON.stringify(data));
    msgBuffer.prepend(newMsg);
  };
  clientElem.hidden = false;
}

function handleEvent(event) {
  const data = JSON.parse(event.data);

  for (const msg of data) {
    switch (msg.cmd) {
      case "RoomInfo":
        roomInfo = msg;
        roomInfoConnect();
        break;
      case "DataPackage":
        datapackageConnect(msg);
        break;
      case "Connected":
        connectedConnect(msg);
        break;
      case "ConnectionRefused":
        console.log(msg);
        form.hidden = false;
        break;
      case "PrintJSON":
        printJson(msg);
        break;
      case "ReceivedItems":
        console.log(msg.items);
        const skip = receivedItemIndex - msg.index;
        session.recieved_items(msg.items.slice(skip).map((item) => BigInt(item.item)));
        receivedItemIndex += msg.items.length - skip;
        break;
    }
  }
}

function handleClose() {
  if (!isClosing) {
    window.alert("Connection to the multiserver was lost unexpectedly");
  }
  isClosing = false;
  clientElem.hidden = true;
  for (const elem of clear) elem.innerText = "";
  client = null;
  roomInfo = null;
  datapackageStore = null;
  session = null;
  receivedItemIndex = 0;
  form.hidden = false;
}
