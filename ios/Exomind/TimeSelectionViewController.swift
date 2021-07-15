import UIKit
import SnapKit

class TimeSelectionViewController: ModalGridViewController {
    var value: Date?
    var callback: ((Date?) -> Void)?

    private var choicesView: GridIconsView!
    private var pickerView: UIView!
    private var datePicker: UIDatePicker!

    convenience init(callback: @escaping (Date?) -> Void) {
        self.init()
        self.callback = callback
        self.onClose = { [weak self] in
            self?.callback?(nil)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.initChoicesView()
        self.initPickerView()
        self.showChoicesView()
    }

    private func initChoicesView() {
        let choices = Snoozing.getLaterChoices()
        let gridItems = choices.map {
            choice -> GridIconsViewItem in
            let fa = ObjectsIcon.faIcon(forName: Snoozing.getLaterIcon(choice.key))
            return GridIconsViewItem(label: choice.copy, icon: fa, callback: {
                [weak self] (item) -> () in
                self?.handleChoiceSelection(choice)
            })
        }
        self.choicesView = GridIconsView(items: gridItems)
        self.choicesView.initView()
    }

    private func initPickerView() {
        self.pickerView = UIView()

        datePicker = UIDatePicker()
        datePicker.setValue(UIColor.white, forKey: "textColor")
        datePicker.datePickerMode = .dateAndTime
        datePicker.addTarget(self, action: #selector(handlePickChanged), for: UIControl.Event.valueChanged)
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

    private func showChoicesView() {
        if (self.pickerView.superview != nil) {
            self.pickerView.removeFromSuperview()
        }
        self.view.addSubview(self.choicesView)
        self.choicesView.frame = self.view.frame
    }

    private func showPickerView() {
        if (self.choicesView.superview != nil) {
            self.choicesView.removeFromSuperview()
        }
        self.view.addSubview(self.pickerView)
        self.pickerView.frame = self.view.frame
    }

    private func handleChoiceSelection(_ choice: LaterTimeChoice) {
        let key = choice.key
        if (key == "pick") {
            self.showPickerView()
        } else {
            self.value = Snoozing.textDiffToDate(key)
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

    @objc func handlePickChanged() {
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
