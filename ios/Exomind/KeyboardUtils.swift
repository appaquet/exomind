//
//  KeyboardUtils.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-03-01.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation
import UIKit

class KeyboardUtils: NSObject {
    static let sharedInstance = KeyboardUtils()
    var keyboardShown: Bool = false

    override init() {
        super.init()
        self.addShownObserver(self, selector: #selector(handleKeyboardShown))
        self.addHiddenObserver(self, selector: #selector(handleKeyboardHidden))
    }

    @objc func handleKeyboardShown(_ notification: Notification) {
        self.keyboardShown = true
    }

    @objc func handleKeyboardHidden(_ notification: Notification) {
        self.keyboardShown = false
    }

    func addWillShowObserver(_ observer: AnyObject, selector aSelector: Selector) {
        NotificationCenter.default.addObserver(observer, selector: aSelector, name: UIResponder.keyboardWillShowNotification, object: nil)
    }

    func addShownObserver(_ observer: AnyObject, selector aSelector: Selector) {
        NotificationCenter.default.addObserver(observer, selector: aSelector, name: UIResponder.keyboardDidShowNotification, object: nil)
    }

    func addHiddenObserver(_ observer: AnyObject, selector aSelector: Selector) {
        NotificationCenter.default.addObserver(observer, selector: aSelector, name: UIResponder.keyboardDidHideNotification, object: nil)
    }

    func removeObserver(_ observer: AnyObject) {
        NotificationCenter.default.removeObserver(observer)
    }
}
