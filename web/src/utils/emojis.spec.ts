import Emojis from "./emojis";

describe('Emojis', () => {
    it('should allow testing for emoji prefix', () => {
        expect(Emojis.hasEmojiPrefix('😬')).toBeTruthy();
        expect(Emojis.hasEmojiPrefix('hello 😬')).toBeFalsy();
        expect(Emojis.hasEmojiPrefix('😬 hello')).toBeTruthy();
    });

    it('should allow extracting emoji from prefix', () => {
        expect(Emojis.extractEmojiPrefix('😬')).toEqual(['😬', '']);
        expect(Emojis.extractEmojiPrefix('😬 hello')).toEqual(['😬', 'hello']);
        expect(Emojis.extractEmojiPrefix('hello 😬')).toEqual(['', 'hello 😬']);
    });
});
