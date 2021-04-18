import Emojis from "./emojis";

describe('Emojis', () => {
    it('should allow testing for title starts with emoji', () => {
        expect(Emojis.startsWithEmoji('😬')).toBeTruthy();
        expect(Emojis.startsWithEmoji('hello 😬')).toBeFalsy();
        expect(Emojis.startsWithEmoji('😬 hello')).toBeTruthy();
    });

    it('should allow extracting emoji from collection title', () => {
        expect(Emojis.extractEmoji('😬')).toEqual(['😬', ''])
        expect(Emojis.extractEmoji('😬 hello')).toEqual(['😬', 'hello'])
        expect(Emojis.extractEmoji('hello 😬')).toEqual(['', 'hello 😬'])
    });
});
