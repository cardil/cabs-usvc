import 'package:flutter/widgets.dart';
import 'package:frontend/ride.dart';

class AcceptPage extends StatefulWidget {
  const AcceptPage({super.key, required this.ride});

  final Ride ride;

  @override
  State<AcceptPage> createState() => _AcceptPageState();
}

class _AcceptPageState extends State<AcceptPage> {
  @override
  Widget build(BuildContext context) {
    return ListView(
      children: <Widget>[
        header(context: context, id: widget.ride.id, state: widget.ride.state),
        adressLine(context: context, address: widget.ride.from, label: 'From'),
        adressLine(context: context, address: widget.ride.to, label: 'To'),
      ],
    );
  }

  header({required BuildContext context, required int id, required RideState state}) {}
}
