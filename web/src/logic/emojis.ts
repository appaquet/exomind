
import emojiRegex from "emoji-regex";
const emojiRe = emojiRegex();

export default class Emojis {
    static hasEmojiPrefix(str: string): boolean {
        const match = str.match(emojiRe);
        if (!match) {
            return false;
        }

        const emoji = match[0];
        return str.startsWith(emoji);
    }

    static extractEmojiPrefix(str: string): [string, string] {
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

    static stripEmojiPrefix(title: string): string {
        const [, ext] = Emojis.extractEmojiPrefix(title);
        return ext;
    }
}