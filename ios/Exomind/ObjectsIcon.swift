//
//  ObjectsIcon.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-12-14.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import FontAwesome_swift

class ObjectsIcon {
    fileprivate static var iconCache = [String : UIImage]()

    
    static func icon(forEntity: HCEntity, color: UIColor, dimension: CGFloat) -> UIImage {
        let entityTrait = EntityTrait(entity: forEntity)
        let fa = ObjectsIcon.icon(forName: entityTrait?.icon ?? "question")
        return cached(fa: fa, color: color, dimension: dimension, block: { () -> UIImage in
            return UIImage.fontAwesomeIcon(name: fa, style: .solid, textColor: color, size: CGSize(width: dimension, height: dimension))
        })
    }
    
    static func icon(forEntityTrait: EntityTrait, color: UIColor, dimension: CGFloat) -> UIImage {
        let fa = ObjectsIcon.icon(forName: forEntityTrait.icon)
        return cached(fa: fa, color: color, dimension: dimension, block: { () -> UIImage in
            return UIImage.fontAwesomeIcon(name: fa, style: .solid, textColor: color, size: CGSize(width: dimension, height: dimension))
        })
    }
    
    fileprivate static func cached(fa: FontAwesome, color: UIColor, dimension: CGFloat, block: () -> UIImage) -> UIImage {
        let key = "\(fa)_\(color)_\(dimension)"
        if let img = ObjectsIcon.iconCache[key] {
            return img
        } else {
            let img = block()
            ObjectsIcon.iconCache[key] = img
            return img
        }
    }

    static func icon(forName name: String) -> FontAwesome {

        var fa: FontAwesome! = nil
        switch name {
        case "folder-o":
            fa = .folderOpen
        case "inbox":
            fa = .inbox
        case "cloud":
            fa = .cloud
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
        default:
            print("Couldn't find fontawesome icon for \(name)")
            fa = .question
        }
        return fa
    }
}
