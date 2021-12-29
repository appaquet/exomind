import classNames from 'classnames';
import React from 'react';
import Flatpickr from "react-flatpickr";
import "flatpickr/dist/themes/light.css";
import './time-selector.less';
import DateUtil, { SnoozeKey } from '../../../utils/dates';

interface IProps {
    onSelectionDone: (date: Date) => void;
}

interface IState {
    picker: boolean;
    date: Date;
}

export default class TimeSelector extends React.Component<IProps, IState> {
    listElemRef: React.RefObject<HTMLUListElement>;

    constructor(props: IProps) {
        super(props);

        this.state = {
            picker: false,
            date: new Date(),
        };

        this.listElemRef = React.createRef();
    }

    render(): React.ReactNode {
        const body = (!this.state.picker) ? this.renderList() : this.renderPicker();
        const classes = classNames({
            'time-selector': true,
            choices: !this.state.picker,
            picker: !!this.state.picker
        });

        return (
            <div className={classes}>
                <div className="time-selector-header">Snooze until...</div>
                {body}
            </div>
        );
    }

    private renderList(): React.ReactNode {
        return (
            <ul ref={this.listElemRef}>
                {this.renderLaterChoices()}
            </ul>
        );
    }

    private renderLaterChoices(): React.ReactNode {
        return DateUtil.getSnoozeChoices()
            .map((choice) => {
                return (
                    <li onClick={this.handleTimeClick.bind(this, choice.key)} key={choice.key}>
                        <span>{choice.copy}</span>
                    </li>
                )
            });
    }


    private renderPicker(): React.ReactNode {
        return (
            <div>
                <div className="field">
                    <Flatpickr
                        data-enable-time
                        value={this.state.date}
                        onChange={this.handlePickerChange.bind(this)}
                    />
                </div>
                <div className="button">
                    <button onClick={this.handlePickerDone.bind(this)}>
                        <span>Done</span>
                    </button>
                </div>
            </div>
        );
    }

    private handlePickerChange(dates: Date[]): void {
        this.setState({ date: dates[0] });
    }

    private handlePickerDone(): void {
        if (this.props.onSelectionDone && this.state.date) {
            this.props.onSelectionDone(this.state.date);
        }
    }

    private handleTimeClick(key: SnoozeKey): void {
        if (key === 'pick') {
            this.setState({
                picker: true
            });
        } else {
            const date = DateUtil.snoozeDate(key);
            if (this.props.onSelectionDone) {
                this.props.onSelectionDone(date);
            }
        }
    }
}

