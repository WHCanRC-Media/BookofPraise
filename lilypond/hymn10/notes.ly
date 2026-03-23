melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c4 f4 f4 a4 a4 g4 f4 g4 \break

  % Line 2
  a4 g4 f4 a4 g4 f2 \break

  % Line 3
  g4 a4 g4 f4 a4 c8( bes8) a8( g8) a4 \break

  % Line 4
  c4 c2 c2 d2 c4( bes4) c2 \break

  % Line 5
  a4 c4 a4 f4 a4 g8( f8) g8( a8) g4 \break

  % Line 6
  f4 c'2 bes2 a4( bes8 g4) g4 f2 \bar "|."
}
