
import emojiRegex from "emoji-regex";
const emojiRe = emojiRegex();

export default class Emojis {
    static startsWithEmoji(str: string): boolean {
        const match = str.match(emojiRe);
        if (!match) {
            return false;
        }

        const emoji = match[0];
        return str.startsWith(emoji);
    }

    static extractEmoji(str: string): [string, string] {
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
        const [, ext] = Emojis.extractEmoji(title);
        return ext;
    }
}