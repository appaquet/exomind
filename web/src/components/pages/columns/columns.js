import classNames from 'classnames';
import PropTypes from 'prop-types';
import React from 'react';
import { SelectedItem, Selection } from '../../objects/entity-list/selection';
import Column from './column';
import { ColumnConfig, ColumnsConfig } from './columns-config';
import './columns.less';

export default class Columns extends React.Component {
    static propTypes = {
        config: PropTypes.string.isRequired,
        onConfigChange: PropTypes.func.isRequired
    };

    getConfig(props) {
        props = props || this.props;
        var config = ColumnsConfig.fromString(props.config);
        if (config.empty) {
            config = ColumnsConfig.forInbox();
        }
        return config;
    }

    render() {
        let renderedColumns = this.renderColumns();
        let nbColumns = renderedColumns.length;
        let classes = `columns count-${nbColumns}`;
        return (
            <div className={classes}>
                {renderedColumns}
            </div>
        );
    }

    renderColumns() {
        let config = this.getConfig();
        return config.parts.map((columnConfig, colId) => {
            let selectionItems = [];
            let nextColumnConfig = config.parts[colId + 1];
            if (nextColumnConfig) {
                if (nextColumnConfig.isEntity) {
                    selectionItems = [SelectedItem.fromEntityId(nextColumnConfig.value)];
                } else if (nextColumnConfig.isTrait) {
                    selectionItems = [SelectedItem.fromEntityTraitId(nextColumnConfig.value, nextColumnConfig.extra)];
                }
            }
            let selection = new Selection(selectionItems);

            let colKey = `column-container-${colId}`;
            let classes = classNames({
                'column-container': true,
                [colKey]: true
            });

            return (
                <div className={classes} key={colKey}>
                    <Column
                        columnId={colId}
                        columnConfig={columnConfig}
                        key={columnConfig.value}
                        onColumnConfigChange={this.handleColumnConfigChange.bind(this, colId)}

                        selection={selection}
                        onSelectionChange={this.handleColumnItemSelect.bind(this, colId)}
                        onClose={(colId !== 0) ? this.handleColumnClose.bind(this, colId) : null}
                    />
                </div>);
        });
    }

    handleColumnItemSelect(colId, objs) {
        let columnsConfig = this.getConfig();
        if (objs && !objs.isEmpty()) {
            // TODO: support for multiple selections + entity traits
            let firstSel = objs.items[0];

            let columnConfig;
            if (firstSel.traitId) {
                columnConfig = ColumnConfig.forTrait(firstSel.entityId, firstSel.traitId);
            } else {
                columnConfig = ColumnConfig.forEntity(firstSel.entityId);
            }
            columnsConfig = columnsConfig.set(colId + 1, columnConfig);

        } else {
            columnsConfig = columnsConfig.unset(colId + 1);
        }

        this.props.onConfigChange(columnsConfig);
    }

    handleColumnClose(colId) {
        let columnsConfig = this.getConfig().pop(colId);
        this.props.onConfigChange(columnsConfig);
    }

    handleColumnConfigChange(colId, newColumnConfig) {
        let curColumnsConfig = this.getConfig();
        let newColumnsConfig = curColumnsConfig.set(colId, newColumnConfig.toString());
        this.props.onConfigChange(newColumnsConfig);
    }
}

