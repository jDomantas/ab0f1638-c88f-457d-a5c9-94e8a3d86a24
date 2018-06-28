import {test} from "test";

const world = "🗺️";

export function hello(word: string = world): string {
    return `Hello ${world}! ` + test(world);
}
