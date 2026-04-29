melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 a2 bes4 c2 bes4 a2 g4 a2 \break

  % Line 2
  a4 a2 g4 bes4( a4) g4 f2( e4) f2 \break

  % Line 3
  f4 a2 bes4 c2 bes4 a2 g4 a2 \break

  % Line 4
  a4 a2 g4 bes4( a4) g4 f2( e4) f2 \break

  % Line 5
  f4 f2 g4 bes2 a4 g2 f4 g2 \break

  % Line 6
  g4 a2 bes4 c2 bes4 a2 g4 a2 \break

  % Line 7
  f4 g2 bes4 a2 g4 f2( e4) f2 \bar "|."
}
