//
//  TimeSelectionViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-27.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit
import SnapKit

class TimeSelectionViewController: ModalGridViewController {
    var value: Date?
    var callback: ((Date?) -> Void)?

    fileprivate var choicesView: GridIconsView!
    fileprivate var pickerView: UIView!
    fileprivate var datePicker: UIDatePicker!

    convenience init(callback: @escaping ((Date?) -> Void)) {
        self.init()
        self.callback = callback
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.initChoicesView()
        self.initPickerView()
        self.showChoicesView()
    }

    func initChoicesView() {
        let choices = TimeLogic.getLaterChoices()
        let gridItems = choices.map {
            choice -> GridIconsViewItem in
            let fa = ObjectsIcon.icon(forName: TimeLogic.getLaterIcon(choice.key))
            return GridIconsViewItem(label: choice.copy, icon: fa, callback: {
                [weak self] (item) -> () in
                self?.handleChoiceSelection(choice)
            })
        }
        self.choicesView = GridIconsView(items: gridItems)
        self.choicesView.initView()
    }

    func initPickerView() {
        self.pickerView = UIView()

        datePicker = UIDatePicker()
        datePicker.setValue(UIColor.white, forKey: "textColor")
        datePicker.datePickerMode = .dateAndTime
        datePicker.addTarget(self, action: #selector(handlePickeChanged), for: UIControl.Event.valueChanged)
        pickerView.addSubview(datePicker)

        datePicker.snp.makeConstraints { (make) in
            make.center.equalTo(pickerView.snp.center)
        }

        let doneButton = UIButton()
        doneButton.setTitle("Done", for: UIControl.State())
        doneButton.setTitleColor(UIColor.white, for: UIControl.State())
        doneButton.addTarget(self, action: #selector(handlePickDone), for: UIControl.Event.touchUpInside)
        pickerView.addSubview(doneButton)
        doneButton.snp.makeConstraints { (make) in
            make.top.equalTo(datePicker.snp.bottom).offset(10)
            make.centerX.equalTo(pickerView.snp.centerX)
        }
    }

    func showChoicesView() {
        if (self.pickerView.superview != nil) {
            self.pickerView.removeFromSuperview()
        }
        self.view.addSubview(self.choicesView)
        self.choicesView.frame = self.view.frame
    }

    func showPickerView() {
        if (self.choicesView.superview != nil) {
            self.choicesView.removeFromSuperview()
        }
        self.view.addSubview(self.pickerView)
        self.pickerView.frame = self.view.frame
    }

    func handleChoiceSelection(_ choice: LaterTimeChoice) {
        let key = choice.key
        if (key == "pick") {
            self.showPickerView()
        } else {
            self.value = TimeLogic.textDiffToDate(key)
            self.callback?(self.value)
            self.close()
        }
    }

    override func viewDidLayoutSubviews() {
        if (self.pickerView.superview != nil) {
            self.pickerView.frame = self.view.frame
        }
        if (self.choicesView.superview != nil) {
            self.choicesView.frame = self.view.frame
        }
    }

    @objc func handlePickeChanged() {
        self.value = self.datePicker.date
    }

    @objc func handlePickDone() {
        self.callback?(self.value)
        self.close()
    }

    deinit {
        print("TimeSelectionViewController > Deinit")
    }
}
