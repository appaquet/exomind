import Foundation
import FontAwesome_swift

class ObjectsIcon {
    private static var iconCache = [String: UIImage]()

    static func icon(forName: String, color: UIColor, dimension: CGFloat) -> UIImage {
        let fa = ObjectsIcon.faIcon(forName: forName)
        return icon(forFontAwesome: fa, color: color, dimension: dimension)
    }

    static func icon(forAnyTrait: AnyTraitInstance, color: UIColor, dimension: CGFloat) -> UIImage {
        if let typeInstance = forAnyTrait.typeInstance(),
           case let .collection(col) = typeInstance,
           col.message.name.startsWithEmoji() {

            // special case for collections where we use an emoji as an image if the collection name starts with one
            let (emoji, _) = col.message.name.splitFirstEmoji()
            return emoji.textToImage(ofSize: dimension)

        } else {
            let fa = ObjectsIcon.faIcon(forName: forAnyTrait.constants?.icon ?? "question")
            return icon(forFontAwesome: fa, color: color, dimension: dimension)
        }
    }

    static func icon(forFontAwesome: FontAwesome, color: UIColor, dimension: CGFloat) -> UIImage {
        cached(fa: forFontAwesome, color: color, dimension: dimension, block: { () -> UIImage in
            UIImage.fontAwesomeIcon(name: forFontAwesome, style: .solid, textColor: color, size: CGSize(width: dimension, height: dimension))
        })
    }

    static func faIcon(forName name: String) -> FontAwesome {
        var fa: FontAwesome! = nil
        switch name {
        case "folder-o":
            fa = .folderOpen
        case "inbox":
            fa = .inbox
        case "cloud":
            fa = .cloud
        case "clock-o":
            fa = .clock
        case "chevron-right":
            fa = .chevronRight
        case "search":
            fa = .search
        case "pencil":
            fa = .pen
        case "envelope-o":
            fa = .envelopeOpen
        case "bars":
            fa = .bars
        case "link":
            fa = .link
        case "check-square-o":
            fa = .checkSquare
        case "moon-o":
            fa = .moon
        case "hourglass-start":
            fa = .hourglassStart
        case "history":
            fa = .history
        case "coffee":
            fa = .coffee
        case "soccer-ball-o":
            fa = .baseballBall
        case "briefcase":
            fa = .briefcase
        case "calendar":
            fa = .calendar
        case "calendar-plus-o":
            fa = .calendarPlus
        case "plug":
            fa = .plug
        case "star":
            fa = .star
        case "plus-square":
            fa = .plusSquare
        default:
            print("Couldn't find fontawesome icon for \(name)")
            fa = .question
        }
        return fa
    }

    static func clearCache() {
        ObjectsIcon.iconCache.removeAll()
    }

    private static func cached(fa: FontAwesome, color: UIColor, dimension: CGFloat, style: UIUserInterfaceStyle = .unspecified, block: () -> UIImage) -> UIImage {
        let key = "\(fa)_\(color.hexString())_\(dimension)"
        if let img = ObjectsIcon.iconCache[key] {
            return img
        } else {
            let img = block()
            ObjectsIcon.iconCache[key] = img
            return img
        }
    }
}
