//
//  Stylesheet.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-19.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import UIKit
import UIColor_Hex_Swift

class Stylesheet {
    static let exomindGreen = UIColor("#1D7181")
    static let exomindOrange = UIColor("#D17D2C")
    static let exomindYellow = UIColor("#D19E2C")

    static let navigationBarBg = exomindGreen
    static let navigationBarFg = UIColor.white
    static let navigationBarActiveFg = exomindOrange
    static let tabBarSelectedFg = exomindGreen

    static let collectionSwipeDoneBg = UIColor("#5CB296")
    static let collectionSwipeInboxBg = UIColor("#5CB296")
    static let collectionSwipeLaterBg = exomindYellow
    static let collectionSwipeAddCollectionBg = exomindOrange

    static let collectionSelectorNavigationBarBg = exomindOrange
    static let collectionSelectorNavigationBarFg = navigationBarFg

    static let collectionThemeDoneBg = collectionSwipeDoneBg.withAlphaComponent(0.3)
    static let collectionThemeLaterBg = collectionSwipeLaterBg.withAlphaComponent(0.3)
    static let collectionThemeIconFg = UIColor.white.withAlphaComponent(0.6)
    static let collectionThemeIconSize = 150

    static let searchNavigationBarBg = exomindOrange
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

    static let addButtonBg = exomindYellow
    static let addButtonFg = UIColor.white
    static let addButtonSize = CGFloat(40)

    static let quickButtonBg = exomindGreen
    static let quickButtonAlphaOpened = CGFloat(1.0)
    static let quickButtonAlphaClosed = CGFloat(0.3)
    static let quickButtonFg = exomindYellow
    static let quickButtonSize = CGFloat(50)

    static let quickSecondaryBg = objectColor1
    static let quickSecondaryFg = UIColor.white
    static let quickSecondarySize = CGFloat(35)
    static let quickSecondaryImgSize = CGFloat(25)
    static let quickSecondaryDistance = 60.0

    static let switcherButtonBorderFg = exomindGreen
    static let switcherButtonActiveBg = exomindGreen
    static let switcherButtonInactiveBg = UIColor.white

    static func styleNavigationBar(_ navigationBar: UINavigationBar, bgColor: UIColor, fgColor: UIColor) {
        navigationBar.backgroundColor = bgColor
        navigationBar.barTintColor = bgColor
        navigationBar.tintColor = fgColor

        if (navigationBar.titleTextAttributes == nil) {
            navigationBar.titleTextAttributes = [NSAttributedString.Key: Any]()
        }
        navigationBar.titleTextAttributes![NSAttributedString.Key.foregroundColor] = fgColor
        navigationBar.barStyle = UIBarStyle.black // misleading, it's actually going to be white...
    }
}
