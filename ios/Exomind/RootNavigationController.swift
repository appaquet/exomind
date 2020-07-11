//
//  RootNavigationController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-01.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class RootNavigationController: UINavigationController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    static func mainInstance() -> RootNavigationController? {
        let app = UIApplication.shared
        let vc = app.windows[0].rootViewController as? RootNavigationController
        return vc
    }

    override func viewDidLoad() {
        SessionStore.onChange {
            [weak self]() -> () in
            if let this = self {
                NSObject.cancelPreviousPerformRequests(withTarget: this)
                this.perform(#selector(this.checkViewStatus), with: nil, afterDelay: 0.5)
            }
        }
        self.perform(#selector(checkViewStatus), with: nil, afterDelay: 1.0)
    }

    @objc func checkViewStatus() {
        if (DomainStore.instance.unauthorized()) {
            self.showLogin()
        } else {
            let onTabBar = self.topViewController as? TabBarController != nil
            if (DomainStore.instance.connected() || onTabBar) {
                self.showTabBar()
            } else {
                self.showSplash()
            }
        }
        self.perform(#selector(checkViewStatus), with: nil, afterDelay: 5.0)
    }

    func showTabBar() {
        let onTabBar = self.topViewController as? TabBarController != nil
        if (!onTabBar) {
            self.popToRootViewController(animated: false)
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "tabBarViewController")
            self.show(vc, sender: self)
            NotificationsController.maybeRegister()
        }
    }

    func showLogin() {
        let onLogin = self.topViewController as? LoginViewController != nil
        if (!onLogin) {
            self.popToRootViewController(animated: false)
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "loginViewController")
            self.show(vc, sender: self)
        }
    }

    func showSplash() {
        let onSplash = self.topViewController as? SplashViewController != nil
        if (!onSplash) {
            self.popToRootViewController(animated: false)
        }
    }
    
    func show(navigationObject: NavigationObject) {
        if let tabBar = self.topViewController as? TabBarController {
            tabBar.show(navigationObject: navigationObject)
        }
    }
    
}
