import 'package:flutter/material.dart';

class PropertyCard extends StatelessWidget {
  final Widget child;
  final double? minWidth;
  final double? maxWidth;
  final Widget? title;
  final Widget? subtitle;

  const PropertyCard({
    Key? key,
    required this.child,
    this.minWidth,
    this.maxWidth,
    this.title,
    this.subtitle,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) => ConstrainedBox(
        constraints: BoxConstraints(
          minWidth: minWidth ?? 0.0,
          maxWidth: maxWidth ?? double.infinity,
        ),
        child: Card(
          elevation: 8,
          child: Material(
            type: MaterialType.transparency,
            child: IntrinsicWidth(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (subtitle != null || title != null)
                    ListTile(
                      title: title,
                      subtitle: subtitle,
                    ),
                  Padding(
                    padding: EdgeInsets.only(
                      top: subtitle == null && title == null ? 16 : 0,
                      left: 16,
                      right: 16,
                      bottom: 16,
                    ),
                    child: child,
                  ),
                ],
              ),
            ),
          ),
        ),
      );
}
