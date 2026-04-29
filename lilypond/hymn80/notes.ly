melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  bes'4 bes4( a4) g4 f2 bes4 c4( bes4) a4 bes2 \break

  % Line 2
  c4 d4( c4) bes4 a4( bes4) c4 bes4( a4) g4 f2 \break

  % Line 3
  f4 g4( f4) g8( a8) bes2 a4 bes2 c4 d2 \break

  % Line 4
  c8( d8) ees4( d4) c4 bes4( a4) bes4 c4( bes4) a4 bes2 \bar "|."
}
