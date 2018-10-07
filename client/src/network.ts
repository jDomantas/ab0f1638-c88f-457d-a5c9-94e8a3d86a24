import { w3cwebsocket as WebSocketClient } from "websocket";

type ServerMessage = WorldStateMessage | PlayerInputMessage;

export interface WorldStateMessage {
    frame: number;
    world: string;
}

export interface PlayerInputs {
    [id: string]: string;
}

export interface PlayerInputMessage {
    frame: number;
    new_players: number[];
    removed_players: number[];
    inputs: PlayerInputs;
}

export interface LocalPlayerInput {
    frame: number;
    input: string;
}

export class NetworkHandler {
    public onWorldState: (world: WorldStateMessage) => void;
    public onPlayerInputs: (inputs: PlayerInputMessage) => void;
    private client: WebSocketClient;
    private pendingInputs: PlayerInputMessage[];
    private receivedWorldState: boolean;

    constructor() {
        this.client = new WebSocketClient("ws://" + location.host + "/ws");
        this.pendingInputs = [];
        this.receivedWorldState = false;
        this.onWorldState = _ => {};
        this.onPlayerInputs = _ => {};
        this.client.onopen = () => this.onOpen();
        this.client.onerror = err => this.error(err);
        this.client.onclose = () => this.onClose();
        this.client.onmessage = msg => this.onMessage(msg);
    }

    public sendInput(input: LocalPlayerInput) {
        this.client.send(JSON.stringify(input));
    }

    private error(err: Error) {
        console.error(`Connection error: ${err}`);
    }

    private onOpen() {
        console.info("Connected");
    }

    private onClose() {
        console.info("Disconnected");
    }

    private onMessage(message: any) {
        console.debug("Received message:", message);
        const payload = parseMessagePayload(message);
        if (isWorldState(payload)) {
            this.receivedWorldState = true;
            this.onWorldState(payload);
            this.pendingInputs.forEach(this.onPlayerInputs);
            this.pendingInputs = [];
        } else {
            if (this.receivedWorldState) {
                this.onPlayerInputs(payload);
            } else {
                this.pendingInputs.push(payload);
            }
        }
    }
}

function parseMessagePayload(message: any): ServerMessage {
    return JSON.parse(message.data) as ServerMessage;
}

function isWorldState(message: ServerMessage): message is WorldStateMessage {
    const m = message as WorldStateMessage;
    return m.frame !== undefined && m.world !== undefined;
}
