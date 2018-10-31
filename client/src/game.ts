import { Handle, InputHandle, LowLevelGame, WorldHandle } from "lowLevelGame";

export class Game {
    private readonly game: LowLevelGame;

    public constructor(wasmInstance: WebAssembly.Instance) {
        this.game = new LowLevelGame(wasmInstance);
    }

    public updatePlayer(world: World, player: PlayerId, input: Input): World {
        const handle = this.game.updatePlayer(world.handle, player.id, input.handle);
        return new World(this.game, handle);
    }

    public updateWorld(world: World): World {
        const handle = this.game.updateWorld(world.handle);
        return new World(this.game, handle);
    }

    public addPlayer(player: PlayerId, world: World): World {
        const handle = this.game.addPlayer(world.handle, player.id);
        return new World(this.game, handle);
    }

    public removePlayer(player: PlayerId, world: World): World {
        const handle = this.game.removePlayer(world.handle, player.id);
        return new World(this.game, handle);
    }

    public deserializeInput(raw: Uint8Array): Input {
        const buffer = this.game.allocateBuffer(raw.length);
        const ptr = this.game.bufferPtr(buffer);
        this.game.writeMemory(ptr, raw);
        const inputHandle = this.game.deserializeInput(buffer);
        this.game.freeHandle(buffer);
        return new Input(this.game, inputHandle);
    }

    public deserializeWorld(raw: Uint8Array): World {
        const buffer = this.game.allocateBuffer(raw.length);
        const ptr = this.game.bufferPtr(buffer);
        this.game.writeMemory(ptr, raw);
        const worldHandle = this.game.deserializeWorld(buffer);
        this.game.freeHandle(buffer);
        return new World(this.game, worldHandle);
    }

    public serializeInput(input: Input): Uint8Array {
        const buffer = this.game.serializeInput(input.handle);
        const ptr = this.game.bufferPtr(buffer);
        const size = this.game.bufferSize(buffer);
        const raw = this.game.readMemory(ptr, size);
        this.game.freeHandle(buffer);
        return raw;
    }

    public createInput(letters: number, oldLetters: number, other: number, oldOther: number): Input {
        const handle = this.game.createInput(letters, oldLetters, other, oldOther);
        return new Input(this.game, handle);
    }
}

export class Input {
    public readonly game: LowLevelGame;
    public readonly handle: InputHandle;

    constructor(game: LowLevelGame, handle: InputHandle) {
        this.game = game;
        this.handle = handle;
    }

    public free() {
        this.game.freeHandle(this.handle);
    }
}

export class PlayerId {
    public readonly id: number;

    public constructor(id: number) {
        this.id = id;
    }
}

export class World {
    public readonly game: LowLevelGame;
    public readonly handle: WorldHandle;

    constructor(game: LowLevelGame, handle: WorldHandle) {
        this.game = game;
        this.handle = handle;
    }

    public free() {
        this.game.freeHandle(this.handle);
    }
}
