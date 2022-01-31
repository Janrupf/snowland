import 'package:flutter/material.dart';

class ModuleSelector extends StatelessWidget {
  final String title;
  final IconData icon;
  final VoidCallback onPressed;

  const ModuleSelector({
    Key? key,
    required this.title,
    required this.icon,
    required this.onPressed,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) => Card(
        elevation: 8.0,
        margin: const EdgeInsets.all(16.0),
        child: InkWell(
          onTap: onPressed,
          child: SizedBox(
            width: 200,
            height: 200,
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(
                    title,
                    style: Theme.of(context).textTheme.headline5,
                  ),
                  Icon(
                    icon,
                    size: 100,
                  )
                ],
              ),
            ),
          ),
        ),
      );
}
