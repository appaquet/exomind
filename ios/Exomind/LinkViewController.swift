//
//  LinkViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-01-19.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit
import SafariServices
import WebKit

class LinkViewController: UIViewController, EntityTraitView {
    @IBOutlet weak var webView: WKWebView!

    fileprivate var entityTrait: EntityTrait!

    func loadEntityTrait(_ entityTrait: EntityTrait) {
        self.entityTrait = entityTrait
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()

        if let link = self.entityTrait.trait as? Link, let url = URL(string: link.url) {
            let request = URLRequest(url: url)
            self.webView.load(request)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
    }

    deinit {
        print("LinkViewController > Deinit")
    }

}
