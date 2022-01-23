import Foundation
import UIKit
import UIColor_Hex_Swift

class Stylesheet {
    static let exomindPrimary = UIColor("#0F76C4")
    static let exomindSecondary = UIColor("#E09626")
    static let exomindYellow = UIColor("#D19E2C")

    static let navigationBarBg = exomindPrimary
    static let navigationBarFg = exomindPrimary
    static let navigationBarActiveFg = exomindSecondary
    static let tabBarSelectedFg = exomindPrimary

    static let collectionSwipeDoneBg = UIColor("#5CB296")
    static let collectionSwipeInboxBg = UIColor("#5CB296")
    static let collectionSwipeSnoozeBg = UIColor("#2670E0")
    static let collectionSwipeCollectionBg = exomindSecondary
    static let collectionSwipeMoveInboxBg = UIColor("#5CB296")
    static let collectionSwipeMoreBg = UIColor("#C0C0C0")

    static let collectionSelectorNavigationBarBg = exomindSecondary
    static let collectionSelectorNavigationBarFg = navigationBarFg

    static let searchNavigationBarBg = exomindSecondary
    static let searchNavigationBarFg = navigationBarFg

    static let objectColor1 = UIColor("#A9CBD1")
    static let objectColor2 = UIColor("#E9BD9D")
    static let objectColor3 = UIColor("#E7CE94")
    static let objectColor4 = UIColor("#939FC5")
    static let objectColor5 = UIColor("#A0BA82")
    static let objectColor6 = UIColor("#A4C7F0")
    static let objectColor7 = UIColor("#C5AB94")
    static let objectColor8 = UIColor("#A7F1E0")
    static let objectColor9 = UIColor("#E3C4FE")
    static let objectColor10 = UIColor("#FEC4C4")
    static let objectColors = [objectColor1, objectColor2, objectColor3, objectColor4, objectColor5, objectColor6, objectColor7, objectColor8, objectColor9, objectColor10]

    static func objectColor(forId: Int) -> UIColor {
        let finalId = forId - 1
        if finalId >= 0 && finalId < objectColors.count {
            return Stylesheet.objectColors[forId - 1]
        } else {
            return objectColor1
        }
    }

    static func objectColor(forString: String) -> UIColor {
        if let int = Int(forString) {
            return objectColor(forId: int)
        } else {
            return UIColor(forString)
        }
    }

    static let quickButtonBg = exomindPrimary
    static let quickButtonAlphaOpened = CGFloat(1.0)
    static let quickButtonAlphaClosed = CGFloat(0.3)
    static let quickButtonSize = CGFloat(50)
    static let quickButtonFg = UIColor.white

    static let quickSecondaryBg = exomindPrimary
    static let quickSecondaryFg = UIColor.white
    static let quickSecondarySize = CGFloat(35)
    static let quickSecondaryImgSize = CGFloat(25)
    static let quickSecondaryDistance = 60.0

    static let switcherButtonBorderFg = exomindPrimary
    static let switcherButtonActiveBg = exomindPrimary
    static let switcherButtonInactiveBg = UIColor.white

    static func styleNavigationBar(_ navigationBar: UINavigationBar, bgColor: UIColor, fgColor: UIColor) {
//        navigationBar.backgroundColor = bgColor
//        navigationBar.barTintColor = bgColor
//        navigationBar.tintColor = fgColor

//        if (navigationBar.titleTextAttributes == nil) {
//            navigationBar.titleTextAttributes = [NSAttributedString.Key: Any]()
//        }
//        navigationBar.titleTextAttributes![NSAttributedString.Key.foregroundColor] = fgColor
//        navigationBar.barStyle = UIBarStyle.black // misleading, it's actually going to be white...
    }

    static func styleSearchBar(_ searchBar: UISearchBar, bgColor: UIColor, fgColor: UIColor) {
//        searchBar.isTranslucent = false
//
//        // Change color of placeholder text
//        // From https://stackoverflow.com/questions/11827585/uisearchbar-change-placeholder-color
//        let textFieldInsideSearchBar = searchBar.value(forKey: "searchField") as? UITextField
//        if let textFieldInsideSearchBar = textFieldInsideSearchBar {
//            textFieldInsideSearchBar.textColor = fgColor
//            let textFieldInsideSearchBarLabel = textFieldInsideSearchBar.value(forKey: "placeholderLabel") as? UILabel
//            textFieldInsideSearchBarLabel?.textColor = fgColor
//        }
    }
}
