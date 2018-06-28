import { w3cwebsocket as WebSocketClient } from "websocket";

const client = new WebSocketClient("ws://localhost:8000/ws");

client.onerror = error => {
    console.log("Connection failed", error);
};

client.onopen = () => {
    console.log("Connected");
    const sayHello = () => {
        if (client.readyState === client.OPEN) {
            client.send("Hello from client!");
        }
    };
    sayHello();
};

client.onclose = () => {
    console.log(`Connection closed`);
};

client.onmessage = message => {
    console.log("Received message", message);
};
