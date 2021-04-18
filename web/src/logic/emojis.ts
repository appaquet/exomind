
import { exomind } from "../protos";
import emojiRegex from "emoji-regex";
const emojiRe = emojiRegex();

export default class Emojis {
    static startsWithEmoji(collection: exomind.base.ICollection): boolean {
        const match = collection.name.match(emojiRe);
        if (!match) {
            return false;
        }

        const emoji = match[0];
        return collection.name.startsWith(emoji);
    }

    static extractEmoji(collection: exomind.base.ICollection): [string, string] {
        return Emojis.extractEmojiStr(collection.name);
    }

    static extractEmojiStr(str: string): [string, string] {
        const match = str.match(emojiRe);
        if (!match) {
            return ['', str];
        }

        const emoji = match[0];
        if (!str.startsWith(emoji)) {
            return ['', str];
        }

        const remain = str.slice(emoji.length).trim();
        return [match[0], remain];
    }

    static stripEmoji(title: string): string {
        const [, ext] = Emojis.extractEmojiStr(title);
        return ext;
    }
}