import UIKit

extension String {
    func nonEmpty() -> String? {
        if !self.isEmpty {
            return self
        } else {
            return nil
        }
    }

    // See https://stackoverflow.com/questions/38809425/convert-apple-emoji-string-to-uiimage
    func textToImage(ofSize: CGFloat) -> UIImage {
        let nsString = (self as NSString)
        let font = UIFont.systemFont(ofSize: ofSize)
        let stringAttributes = [NSAttributedString.Key.font: font]
        let imageSize = nsString.size(withAttributes: stringAttributes)

        UIGraphicsBeginImageContextWithOptions(imageSize, false, 0)
        UIColor.clear.set()
        UIRectFill(CGRect(origin: CGPoint(), size: imageSize))
        nsString.draw(at: CGPoint.zero, withAttributes: stringAttributes)
        let image = UIGraphicsGetImageFromCurrentImageContext()
        UIGraphicsEndImageContext()

        return image ?? UIImage()
    }

    func containsEmoji() -> Bool {
        for character in self {
            if character.isEmoji {
                return true
            }
        }
        return false
    }

    func startsWithEmoji() -> Bool {
        self.first?.isEmoji ?? false
    }

    func splitFirstEmoji() -> (String, String) {
        var emoji = String()
        var rest = String()

        for (i, character) in self.enumerated() {
            if i == 0 && character.isEmoji {
                emoji.append(character)
            } else {
                rest.append(character)
            }
        }

        return (emoji, rest.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines))
    }
}


extension Character {
    // From https://stackoverflow.com/questions/30757193/find-out-if-character-in-string-is-emoji
    // An emoji can either be a 2 byte unicode character or a normal UTF8 character with an emoji modifier
    // appended as is the case with 3️⃣. 0x238C is the first instance of UTF16 emoji that requires no modifier.
    // `isEmoji` will evaluate to true for any character that can be turned into an emoji by adding a modifier
    // such as the digit "3". To avoid this we confirm that any character below 0x238C has an emoji modifier attached
    var isEmoji: Bool {
        guard let scalar = unicodeScalars.first else {
            return false
        }
        return scalar.properties.isEmoji && (scalar.value > 0x238C || unicodeScalars.count > 1)
    }
}