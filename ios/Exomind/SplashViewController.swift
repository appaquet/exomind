//
//  SplashViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-01.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class SplashViewController: UIViewController {
    @IBAction func skipLogin(_ sender: AnyObject) {
        (self.navigationController as? RootNavigationController)?.showLogin()
    }
}
