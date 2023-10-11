import 'package:flutter/widgets.dart';
import 'package:frontend/ride.dart';

class OffersPage extends StatefulWidget {
  const OffersPage({super.key, required this.rides});

  final List<Ride> rides;

  @override
  State<OffersPage> createState() => _OffersPageState();
}

class _OffersPageState extends State<OffersPage> {
  @override
  Widget build(BuildContext context) {
    return ListView(
      children: widget.rides.map((ride) => RideWidget(ride: ride)).toList(),
    );
  }
}
