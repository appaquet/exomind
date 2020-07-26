//
//  TaskViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-01-19.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class TaskViewController: UIViewController, EntityTraitViewOld {
    fileprivate var entityTrait: EntityTraitOld!
    fileprivate var changed: Bool = false

    @IBOutlet weak var taskNameField: UITextField!

    func loadEntityTrait(_ entityTrait: EntityTraitOld) {
        self.entityTrait = entityTrait
    }

    override func viewDidLoad() {
        self.taskNameField.text = self.entityTrait.displayName
    }

    override func viewDidAppear(_ animated: Bool) {
        self.taskNameField.becomeFirstResponder()
        self.taskNameField.selectAll(self)
    }

    @IBAction func nameChanged(_ sender: AnyObject) {
        changed = true
    }

    override func viewWillDisappear(_ animated: Bool) {
        if (self.changed && self.taskNameField.text != nil && self.taskNameField.text != "") {
            if let taskTrait = entityTrait.trait as? TaskFull, let text = self.taskNameField.text {
                let newTask = taskTrait.clone() as! TaskFull
                newTask.title = text
                ExomindDSL.on(entityTrait.entity).mutate.put(newTask).execute()
            }
        }
    }
}
