import { exomind } from "../protos";
import Emojis from "./emojis";

describe('Emojis', () => {
    it('should allow testing for title starts with emoji', () => {
        expect(Emojis.startsWithEmoji(new exomind.base.Collection({ name: '😬' }))).toBeTruthy();
        expect(Emojis.startsWithEmoji(new exomind.base.Collection({ name: 'hello 😬' }))).toBeFalsy();
        expect(Emojis.startsWithEmoji(new exomind.base.Collection({ name: '😬 hello' }))).toBeTruthy();
    });

    it('should allow extracting emoji from collection title', () => {
        expect(Emojis.extractEmoji(new exomind.base.Collection({ name: '😬' }))).toEqual(['😬', ''])
        expect(Emojis.extractEmoji(new exomind.base.Collection({ name: '😬 hello' }))).toEqual(['😬', 'hello'])
        expect(Emojis.extractEmoji(new exomind.base.Collection({ name: 'hello 😬' }))).toEqual(['', 'hello 😬'])
    });
});
